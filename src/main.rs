use std::sync::{Arc, Mutex};
use make87_messages::image::uncompressed::ImageRawAny;
use opencv::{core, imgproc, prelude::*, video};

fn yuv_to_gray(yuv_bytes: &[u8], width: i32, height: i32) -> opencv::Result<Mat> {
    let y_plane_size = (width * height) as usize;
    if yuv_bytes.len() < y_plane_size {
        return Err(opencv::Error::new(
            core::StsUnmatchedSizes,
            format!("Invalid YUV buffer size: got {}, expected at least {}", yuv_bytes.len(), y_plane_size),
        ));
    }
    let y_data = &yuv_bytes[..y_plane_size];
    Mat::from_slice(y_data)?.reshape(1, height).map(|mat| mat.clone_pointee())
}

fn rgb888_to_gray(rgb_bytes: &[u8], width: i32, height: i32) -> opencv::Result<Mat> {
    let expected = (width * height * 3) as usize;
    if rgb_bytes.len() != expected {
        return Err(opencv::Error::new(
            core::StsUnmatchedSizes,
            format!("Invalid RGB888 buffer size: got {}, expected {}", rgb_bytes.len(), expected),
        ));
    }
    let mat = Mat::from_slice(rgb_bytes)?;
    let mat = mat.reshape(3, height)?;
    let mut gray = Mat::default();
    imgproc::cvt_color(&mat, &mut gray, imgproc::COLOR_RGB2GRAY, 0)?;
    Ok(gray)
}

fn rgba8888_to_gray(rgba_bytes: &[u8], width: i32, height: i32) -> opencv::Result<Mat> {
    let expected = (width * height * 4) as usize;
    if rgba_bytes.len() != expected {
        return Err(opencv::Error::new(
            core::StsUnmatchedSizes,
            format!("Invalid RGBA8888 buffer size: got {}, expected {}", rgba_bytes.len(), expected),
        ));
    }
    let mat = Mat::from_slice(rgba_bytes)?;
    let mat = mat.reshape(4, height)?;
    let mut gray = Mat::default();
    imgproc::cvt_color(&mat, &mut gray, imgproc::COLOR_RGBA2GRAY, 0)?;
    Ok(gray)
}

fn downsample(mat: &Mat, target_width: i32, target_height: i32) -> opencv::Result<Mat> {
    let mut resized = Mat::default();
    imgproc::resize(
        mat,
        &mut resized,
        core::Size::new(target_width, target_height),
        0.0,
        0.0,
        imgproc::INTER_LINEAR,
    )?;
    Ok(resized)
}

fn detect_motion_mog2(
    background_subtractor: &mut core::Ptr<video::BackgroundSubtractorMOG2>,
    frame: &Mat,
) -> opencv::Result<bool> {
    let mut fg_mask = Mat::default();
    BackgroundSubtractorMOG2Trait::apply(background_subtractor, frame, &mut fg_mask, -1.0)?;
    Ok(core::count_non_zero(&fg_mask)? > 500)
}

fn main() -> opencv::Result<()> {
    make87::initialize();

    let down_width = make87::get_config_value("PROCESSING_RESCALE_WIDTH")
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map_or_else(
            || 960,
            |s| s.parse::<i32>().unwrap_or_else(|e| {
                eprintln!("Failed to parse PROCESSING_RESCALE_WIDTH ('{}'): {}", s, e);
                std::process::exit(1);
            }),
        );

    // MOG2 config with defaults
    let mog2_history = make87::get_config_value("MOG2_HISTORY")
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map_or(500, |s| s.parse::<i32>().unwrap_or_else(|e| {
            eprintln!("Failed to parse MOG2_HISTORY ('{}'): {}", s, e);
            std::process::exit(1);
        }));
    let mog2_var_threshold = make87::get_config_value("MOG2_VAR_THRESHOLD")
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map_or(16.0, |s| s.parse::<f64>().unwrap_or_else(|e| {
            eprintln!("Failed to parse MOG2_VAR_THRESHOLD ('{}'): {}", s, e);
            std::process::exit(1);
        }));
    let mog2_detect_shadows = make87::get_config_value("MOG2_DETECT_SHADOWS")
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map_or(true, |s| match s.to_ascii_lowercase().as_str() {
            "0" | "false" => false,
            _ => true,
        });

    let mog2 = Arc::new(Mutex::new(
        video::create_background_subtractor_mog2(mog2_history, mog2_var_threshold, mog2_detect_shadows)?
    ));

    let publisher_topic_name = make87::resolve_topic_name("MOTION_IMAGE_RAW")
        .expect("Failed to resolve topic name 'MOTION_IMAGE_RAW'");
    let publisher = make87::get_publisher::<ImageRawAny>(publisher_topic_name)
        .expect("Failed to create MOTION_IMAGE_RAW publisher");

    let subscriber_topic_name = make87::resolve_topic_name("IMAGE_RAW")
        .expect("Failed to resolve topic name 'IMAGE_RAW'");

    if let Some(topic) = make87::get_subscriber::<ImageRawAny>(subscriber_topic_name) {
        let mog2 = mog2.clone();
        topic.subscribe(move |message| {
            let (curr_gray, width, height) = match &message.image {
                Some(make87_messages::image::uncompressed::image_raw_any::Image::Yuv420(yuv)) => {
                    yuv_to_gray(&yuv.data, yuv.width as i32, yuv.height as i32)
                        .map(|mat| (mat, yuv.width as i32, yuv.height as i32))
                        .map_err(|e| format!("YUV420 error: {:?}", e))
                }
                Some(make87_messages::image::uncompressed::image_raw_any::Image::Yuv422(yuv)) => {
                    yuv_to_gray(&yuv.data, yuv.width as i32, yuv.height as i32)
                        .map(|mat| (mat, yuv.width as i32, yuv.height as i32))
                        .map_err(|e| format!("YUV422 error: {:?}", e))
                }
                Some(make87_messages::image::uncompressed::image_raw_any::Image::Yuv444(yuv)) => {
                    yuv_to_gray(&yuv.data, yuv.width as i32, yuv.height as i32)
                        .map(|mat| (mat, yuv.width as i32, yuv.height as i32))
                        .map_err(|e| format!("YUV444 error: {:?}", e))
                }
                Some(make87_messages::image::uncompressed::image_raw_any::Image::Rgb888(rgb)) => {
                    rgb888_to_gray(&rgb.data, rgb.width as i32, rgb.height as i32)
                        .map(|mat| (mat, rgb.width as i32, rgb.height as i32))
                        .map_err(|e| format!("RGB888 error: {:?}", e))
                }
                Some(make87_messages::image::uncompressed::image_raw_any::Image::Rgba8888(rgba)) => {
                    rgba8888_to_gray(&rgba.data, rgba.width as i32, rgba.height as i32)
                        .map(|mat| (mat, rgba.width as i32, rgba.height as i32))
                        .map_err(|e| format!("RGBA8888 error: {:?}", e))
                }
                _ => Err("Unsupported or missing image format in ImageRawAny".to_string()),
            }.unwrap_or_else(|e| {
                eprintln!("{}", e);
                return (Default::default(), 0, 0);
            });

            let down_height = ((height as f32 / width as f32) * down_width as f32).round() as i32;

            let curr_down = match downsample(&curr_gray, down_width, down_height) {
                Ok(mat) => mat,
                Err(e) => {
                    eprintln!("Failed to downsample: {:?}", e);
                    return;
                }
            };

            let mut mog2 = mog2.lock().unwrap();
            if matches!(detect_motion_mog2(&mut *mog2, &curr_down), Ok(true)) {
                println!("Motion detected by background subtraction!");
                if let Err(e) = publisher.publish(&message) {
                    eprintln!("Failed to publish MOTION_IMAGE_RAW: {:?}", e);
                }
            }
        }).expect("Failed to subscribe to IMAGE_RAW");
    }

    make87::keep_running();
    Ok(())
}
