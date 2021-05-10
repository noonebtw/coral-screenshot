use std::{
    cell::RefCell,
    io::{Error, ErrorKind},
    ptr::null,
    rc::Rc,
};

use image::{DynamicImage, GenericImage, GenericImageView, ImageError, ImageResult, Pixel, Rgba};
use x11::{
    xlib::{
        self, IncludeInferiors, True, Window, XCreatePixmap, XDefaultScreen, XGetImage, XGetPixel,
        XGetWindowAttributes, XImage, XOpenDisplay, XPutPixel, XRootWindow, XWindowAttributes,
        ZPixmap,
    },
    xrender::{
        CPSubwindowMode, PictOpOver, PictOpSrc, PictTypeDirect, XRenderColor, XRenderComposite,
        XRenderCreatePicture, XRenderFillRectangle, XRenderFindStandardFormat,
        XRenderFindVisualFormat, XRenderPictureAttributes, XRenderQueryVersion,
    },
};

use crate::traits::ScreenshotBackend;

#[derive(Debug)]
struct Display(Rc<*mut xlib::Display>);

#[derive(Debug, Clone)]
struct XLibPtr<T>(*mut T);

impl<T> XLibPtr<T> {
    unsafe fn new(ptr: *mut T) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }
}

impl<T> Drop for XLibPtr<T> {
    fn drop(&mut self) {
        unsafe {
            xlib::XFree(std::mem::transmute::<_, *mut _>(self.0));
        }
    }
}

#[derive(Debug, Clone)]
pub struct XLibMut<T>(Rc<RefCell<XLibPtr<T>>>);

impl<T> XLibMut<T> {
    pub unsafe fn new(ptr: *mut T) -> Option<Self> {
        Some(Self(Rc::new(RefCell::new(XLibPtr::new(ptr)?))))
    }

    pub fn as_ref(&self) -> &T {
        unsafe { &*self.0.as_ref().borrow().0 }
    }

    pub fn as_mut_ref(&self) -> &mut T {
        unsafe { &mut *self.0.as_ref().borrow().0 }
    }

    #[allow(dead_code)]
    pub unsafe fn as_ptr(&self) -> *const T {
        self.0.as_ref().borrow().0
    }

    pub unsafe fn as_mut_ptr(&self) -> *mut T {
        self.0.as_ref().borrow_mut().0
    }
}

impl Display {
    fn new(dpy: *mut xlib::Display) -> Option<Self> {
        if dpy.is_null() {
            None
        } else {
            Some(Self(Rc::new(dpy)))
        }
    }

    fn get(&self) -> *mut xlib::Display {
        *self.0
    }
}

pub struct XLibState {
    display: Display,
    root: Window,
}

impl XLibState {
    pub fn new() -> Option<Self> {
        let display = Display::new(unsafe { XOpenDisplay(null()) })?;
        let root = unsafe { XRootWindow(display.get(), XDefaultScreen(display.get())) };

        Some(Self { display, root })
    }

    fn dpy(&self) -> *mut xlib::Display {
        self.display.get()
    }

    fn has_xrender(&self) -> bool {
        let mut maj = 0;
        let mut min = 0;

        unsafe { XRenderQueryVersion(self.dpy(), &mut maj, &mut min) == True }
    }

    fn get_window_attributes(&self, window: Window) -> Option<XWindowAttributes> {
        let mut wa = unsafe { std::mem::MaybeUninit::<XWindowAttributes>::zeroed().assume_init() };

        if unsafe { XGetWindowAttributes(self.dpy(), window, &mut wa) != 0 } {
            Some(wa)
        } else {
            None
        }
    }

    fn get_window_dimensions(&self, window: Window) -> Option<(i32, i32)> {
        let wa = self.get_window_attributes(window)?;

        Some((wa.width, wa.height))
    }

    fn get_screenshot_inner(&self, window: Window) -> Option<XLibMut<XImage>> {
        assert_eq!(self.has_xrender(), true);

        let wa = self.get_window_attributes(window)?;

        let format = unsafe { XRenderFindVisualFormat(self.dpy(), wa.visual).as_mut()? };
        let has_alpha = format.type_ == PictTypeDirect && format.direct.alphaMask != 0;
        let mut pa =
            unsafe { std::mem::MaybeUninit::<XRenderPictureAttributes>::zeroed().assume_init() };

        pa.subwindow_mode = IncludeInferiors;

        let picture = unsafe {
            XRenderCreatePicture(
                self.dpy(),
                window,
                format as *mut _,
                CPSubwindowMode as u64,
                &mut pa,
            )
        };

        let (width, height) = self.get_window_dimensions(window)?;
        let (width, height) = (width as u32, height as u32);

        let pixmap = unsafe { XCreatePixmap(self.dpy(), self.root, width, height, 32) };

        // PictStandardARGB32 => 0
        let format2 = unsafe { XRenderFindStandardFormat(self.dpy(), 0).as_mut()? };
        let mut pa2 =
            unsafe { std::mem::MaybeUninit::<XRenderPictureAttributes>::zeroed().assume_init() };

        let pixmap_picture =
            unsafe { XRenderCreatePicture(self.dpy(), pixmap, format2 as *mut _, 0, &mut pa2) };

        let color = XRenderColor {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0,
        };

        let image = unsafe {
            XRenderFillRectangle(
                self.dpy(),
                PictOpSrc,
                pixmap_picture,
                &color,
                0,
                0,
                width,
                height,
            );

            XRenderComposite(
                self.dpy(),
                if has_alpha { PictOpOver } else { PictOpSrc },
                picture,
                0,
                pixmap_picture,
                0,
                0,
                0,
                0,
                0,
                0,
                width,
                height,
            );

            let image = XLibMut::new(XGetImage(
                self.dpy(),
                pixmap,
                0,
                0,
                width,
                height,
                !0u64,
                ZPixmap,
            ))?;

            let img_ref = image.as_mut_ref();

            img_ref.red_mask = (format2.direct.redMask as u64) << format2.direct.red as u64;
            img_ref.green_mask = (format2.direct.greenMask as u64) << format2.direct.green as u64;
            img_ref.blue_mask = (format2.direct.blueMask as u64) << format2.direct.blue as u64;
            img_ref.depth = format2.depth;

            image
        };

        Some(image)
    }
}

impl XLibMut<XImage> {
    fn red_offset(&self) -> u32 {
        self.as_ref().red_mask.trailing_zeros()
    }

    fn green_offset(&self) -> u32 {
        self.as_ref().green_mask.trailing_zeros()
    }

    fn blue_offset(&self) -> u32 {
        self.as_ref().blue_mask.trailing_zeros()
    }

    fn alpha_offset(&self) -> u32 {
        !(self.red_offset() | self.green_offset() | self.blue_offset())
    }
}

impl GenericImageView for XLibMut<XImage> {
    type Pixel = Rgba<u8>;

    type InnerImageView = Self;

    fn dimensions(&self) -> (u32, u32) {
        let self_ref = self.as_ref();

        (self_ref.width as u32, self_ref.height as u32)
    }

    fn bounds(&self) -> (u32, u32, u32, u32) {
        let self_ref = self.as_ref();

        (
            self_ref.xoffset as u32,
            0,
            self_ref.width as u32,
            self_ref.height as u32,
        )
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        let pix = unsafe { XGetPixel(self.as_mut_ptr(), x as i32, y as i32) };

        let self_ref = self.as_ref();
        let alpha_offset = !(self_ref.red_mask | self_ref.green_mask | self_ref.blue_mask);

        let r = (pix & self_ref.red_mask) >> self_ref.red_mask.trailing_zeros();
        let g = (pix & self_ref.green_mask) >> self_ref.green_mask.trailing_zeros();
        let b = (pix & self_ref.blue_mask) >> self_ref.blue_mask.trailing_zeros();
        let a = (pix) >> alpha_offset.trailing_zeros();

        Self::Pixel::from_channels(r as u8, g as u8, b as u8, a as u8)
    }

    fn inner(&self) -> &Self::InnerImageView {
        self
    }
}

impl GenericImage for XLibMut<XImage> {
    type InnerImage = Self;

    fn get_pixel_mut(&mut self, _x: u32, _y: u32) -> &mut Self::Pixel {
        todo!()
    }

    fn put_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel) {
        let mut p = 0u64;
        p &= (pixel[0] << self.red_offset()) as u64;
        p &= (pixel[1] << self.green_offset()) as u64;
        p &= (pixel[2] << self.blue_offset()) as u64;
        p &= (pixel[3] << self.alpha_offset()) as u64;

        unsafe {
            XPutPixel(self.as_mut_ptr(), x as i32, y as i32, p);
        }
    }

    fn blend_pixel(&mut self, _x: u32, _y: u32, _pixel: Self::Pixel) {
        todo!()
    }

    fn inner_mut(&mut self) -> &mut Self::InnerImage {
        self
    }
}

impl ScreenshotBackend for XLibState {
    fn global_screenshot(&self) -> ImageResult<DynamicImage> {
        let ximage = self
            .get_screenshot_inner(self.root)
            .ok_or(ImageError::IoError(Error::new(
                ErrorKind::InvalidData,
                "Failed to aquire screen image",
            )))?;

        let img = ximage
            .view(0, 0, ximage.width(), ximage.height())
            .to_image();

        Ok(DynamicImage::ImageRgba8(img))
    }

    fn window_screenshot(&self, window_id: &str) -> ImageResult<DynamicImage> {
        let window = u64::from_str_radix(window_id.trim_start_matches("0x"), 16).map_err(|_| {
            ImageError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "failed to parse window_id.",
            ))
        })?;

        let ximage = self
            .get_screenshot_inner(window)
            .ok_or(ImageError::IoError(Error::new(
                ErrorKind::InvalidData,
                "Failed to aquire screen image",
            )))?;

        let img = ximage
            .view(0, 0, ximage.width(), ximage.height())
            .to_image();

        Ok(DynamicImage::ImageRgba8(img))
    }
}
