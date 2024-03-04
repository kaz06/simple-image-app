extern crate image as image_crate;
use std::{
    collections::HashMap, error::Error, fmt, hash, path::Path, sync::Arc
};

use image_crate::{
    io::Reader as ImageReader, DynamicImage, ImageBuffer, Pixel, Rgba
};
use iced::widget::image::Handle;


const THRESHOLD: u16 = 65535 / 2;
const MAX_COLOR: u8 = 255;
const FILTER_SIZE: i128 = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProcessType {
    #[default]
    None,
    Binarization,
    ConvolveFilterAVG,
}
impl ProcessType {
    pub const ALL: &'static [Self] = &[
        Self::None,
        Self::Binarization,
        Self::ConvolveFilterAVG,
    ];
}
impl fmt::Display for ProcessType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessType::None => write!(f, "None"),
            ProcessType::Binarization => write!(f, "Binarize"),
            ProcessType::ConvolveFilterAVG => write!(f, "AVG filter"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageProcessError {
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ImagePanelData {
    pub(crate) image: Arc<DynamicImage>,
}

impl ImagePanelData {
    pub fn to_handle(&self) -> iced::widget::image::Handle {
        create_handle(self.image.clone())
        
    }
    pub fn to_rgba8_image_handle(&self) -> iced::widget::image::Handle {
        let binding = self.image.clone().to_rgba8();
        let display_img_buf = ImageBuffer::from_fn(
            self.image.width(), 
            self.image.height(), 
            |x, y| {
            let pixel = binding.get_pixel(x, y);
            Rgba([pixel[0], pixel[1], pixel[2], pixel[3]])
        });
        Handle::from_pixels(self.image.width(), self.image.height(), display_img_buf.into_raw())
    }
    pub fn get_image_width(&self) -> usize {
        self.image.width() as usize
    }
    
    pub fn get_image_height(&self) -> usize {
        self.image.height() as usize
    }

    pub fn get_max_image_value(&self) -> usize {
        *self.image.to_luma16().iter().max().unwrap() as usize
    }

    pub fn get_min_image_value(&self) -> usize {
        *self.image.to_luma16().iter().min().unwrap() as usize
    }
    pub fn get_image(&self) -> Arc<DynamicImage> {
        self.image.clone()
    }
}

#[derive(Debug, Clone, Copy,Eq, PartialEq, hash::Hash)]
pub enum ImageType {
    Original,
    Grayscale,
    Processed,
}


fn create_handle(dynamic_img: Arc<DynamicImage>) -> iced::widget::image::Handle{
    let binding = dynamic_img.to_luma16();
    let display_img_buf = ImageBuffer::from_fn(
        dynamic_img.width(), 
        dynamic_img.height(), 
        |x, y| {
        let pixel = binding.get_pixel(x, y);
        Rgba([(pixel[0]/256) as u8, (pixel[0]/256) as u8, (pixel[0]/256) as u8, 255u8])
    });

    Handle::from_pixels(dynamic_img.width(), dynamic_img.height(), display_img_buf.into_raw())
}

fn create_image_panel_image<P, F, G>(dynamic_img: Arc<DynamicImage>, f: F, g: G) -> Result<ImagePanelData, ImageProcessError> 
where
P: Pixel<Subpixel = u16> + 'static,
F: Fn(Arc<DynamicImage>) -> Arc<ImageBuffer<P, Vec<P::Subpixel>>>,
G: Fn(u32, u32, Arc<ImageBuffer<P, Vec<P::Subpixel>>>) -> Rgba<u8>,
{
    let image_buf = f(dynamic_img.clone());
    let display_img_buf = ImageBuffer::from_fn(
        dynamic_img.width(), 
        dynamic_img.height(), 
        |x, y| {g(x,y, image_buf.clone())});
    Ok(ImagePanelData {
        image: Arc::new(DynamicImage::ImageRgba8(display_img_buf))
    })
}

async fn binarize_image(dynamic_img: Arc<DynamicImage>) -> Result<ImagePanelData, ImageProcessError> {
    create_image_panel_image(
        dynamic_img, 
        |dynamic_img: Arc<DynamicImage>|{Arc::new(dynamic_img.to_luma16())},
        |x, y, image_buf| {
        let pixel = image_buf.get_pixel(x, y);
        if pixel[0] > THRESHOLD {
            Rgba([MAX_COLOR, MAX_COLOR, MAX_COLOR, MAX_COLOR])
        } else {
            Rgba([0, 0, 0, MAX_COLOR])
        }
    })
}

async fn convolve_filter_avg(dynamic_img: Arc<DynamicImage>) -> Result<ImagePanelData, ImageProcessError> {
    create_image_panel_image(
        dynamic_img.clone(), 
        |dynamic_img: Arc<DynamicImage>|{Arc::new(dynamic_img.clone().to_luma16())},
        |x, y, image_buf| {
        let mut filter_weight_sum = 0;
        for i in (-FILTER_SIZE / 2)..(FILTER_SIZE / 2 + 1) {
            for j in (-FILTER_SIZE / 2)..(FILTER_SIZE / 2 + 1) {
                let filter_x = x as i128 + i;
                let filter_y = y as i128 + j;
                if filter_x < 0 || filter_x >= dynamic_img.width() as i128 ||
                 filter_y < 0 || filter_y >= dynamic_img.height() as i128 {
                    continue;
                }
                filter_weight_sum += image_buf.get_pixel(filter_x as u32, filter_y as u32)[0] as u32;
            }
        }
        let luminance_value = ((filter_weight_sum/(FILTER_SIZE*FILTER_SIZE)as u32) / 255) as u8;
        Rgba([luminance_value, luminance_value, luminance_value, MAX_COLOR])   
    })
}

async fn process_none(dynamic_img: Arc<DynamicImage>) -> Result<ImagePanelData, ImageProcessError> {
    Ok(ImagePanelData {
        image: dynamic_img
    })
}

pub fn image_load(images: &mut HashMap<ImageType, ImagePanelData>, path: &str) -> Result<String, Box<dyn Error>>{
    let path = Path::new(path);
    if !path.exists() {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found")));
    }
    
    let load_image = ImageReader::open(path)?.decode()?;
    images.clear();
    images.insert(ImageType::Original, ImagePanelData{image: Arc::new(load_image.clone())});
    images.insert(ImageType::Grayscale, ImagePanelData{image: Arc::new(load_image)});
    Ok("OK".to_string())
}

pub async fn process_image(image_panel_data: ImagePanelData, process_type: ProcessType)  -> Result<ImagePanelData, ImageProcessError>{
    
    match process_type {
        ProcessType::Binarization => binarize_image(image_panel_data.get_image()).await,
        ProcessType::ConvolveFilterAVG => convolve_filter_avg(image_panel_data.get_image()).await,
        ProcessType::None => process_none(image_panel_data.get_image()).await,
    }
}
