use anyhow::Result; // Automatically handle the error types

//use opencv::{self as cv, prelude::*};
use opencv::{
    imgcodecs,
    imgproc,
    highgui,
    videoio,
    prelude::*
};
use opencv::core::Point;
use opencv::core::Rect;
use opencv::core::VecN;
use opencv::core::Mat;
use std::fs;
use std::io;

fn main() -> Result<()> { // Note, this is anyhow::Result
    let directory = "./res/Ghost2";
    let file = "./res/Ghost2/GITS001.bmp";

    //Get the number of files in directory
    let entries = fs::read_dir(directory).unwrap();
    let num_entries = entries.count();
    println!("Number of files: {}", num_entries);
    
    //Get the first file and its dimensions
    let mut image = imgcodecs::imread(file, imgcodecs::IMREAD_GRAYSCALE)?;
    let size = image.size()?;
    let width = size.width;
    let height = size.height;
    println!("Dimensions of files: {}x{}", width, height);

    let mut startX = 170;
    let mut startY = 150;

    let mut endX = 200;
    let mut endY = 180;

    let mut lengthX = endX - startX;
    let mut lengthY = endY - startY;

    let startPoint = Point::new(startX,startY);
    let endPoint = Point::new(endX, endY);
    let color = VecN([250., 2., 250., 0.]);

    imgproc::rectangle(&mut image,
                               Rect::from_points(startPoint,endPoint),  
                               color,
                               1,
                               imgproc::LINE_8,
                               0); 
    
    let mut img_array: Vec<Mat> = vec![image.clone()]; 
    let mut image2 = Mat::default();

    let mut path = "./res/Ghost2/GITS00";
    for i in 2..num_entries+1{
        if i>99 {
           path =  "./res/Ghost2/GITS";
        }
        else if i > 9 {
           path =  "./res/Ghost2/GITS0";
        }

        if i != 2 {
            image = image2.clone();
        }

        let mut current_file = format!("{}{}.bmp",path, i);
        println!("Tracking in image {}",current_file);

        image2 = imgcodecs::imread(&mut current_file, imgcodecs::IMREAD_GRAYSCALE)?;

        let size2 = image2.size()?;
        let height2 = size2.height;
        let width2 = size2.width;
        
        let mut ok = false;
        let mut u = 3; let mut v = 3;

        let mut xStart = 0; let mut yStart = 0;
        let mut xEnd = 0; let mut yEnd = 0;

        while !ok{
            if startX - u > 0 && endX + u < width 
                && startY -v > 0 && endY +v < height {
                    xStart = startX - u;
                    yStart = startY - v;

                    xEnd = endX - u;
                    yEnd = endY - v;
                    ok =true;
                }
            else {
                u = u-1;
                v = v-1;
            }
        }

        let mut SAD = 10000000.0;

        let mut saveOffsetX = 0;
        let mut saveOffsetY = 0;
        let mut start_point = Point::new(startX - 1, startY - 1);
        let mut end_point = Point::new(endX + 1, endY + 1);

        for offsetX in -v..v+2 {
            for offsetY in -u..u+2 {
                let mut val = 0.0;

                let mut avg_1 = 0;
                let mut avg_2 = 0;

                for y in 0..lengthY {
                    for x in 0..lengthX {
                        let val1 = image.at_2d::<u8>(y, x).unwrap();
                        let val2 = image2.at_2d::<u8>(yStart+y+offsetY+v, xStart+x+offsetX+u).unwrap();
                        avg_1 += *val1 as i32;
                        avg_2 += *val2 as i32;
                    }
                }
                avg_1 = avg_1 / (lengthX*lengthY);
                avg_2 = avg_2 / (lengthX*lengthY);

                let mut sigma1 = 0.0;
                let mut sigma2 = 0.0;

                for y in 0..lengthY+2 {
                    for x in 0..lengthX+2 {
                        let val1 = *image.at_2d::<u8>(y, x).unwrap();
                        let val2 = *image2.at_2d::<u8>(yStart+y+offsetY+v, xStart+x+offsetX+u).unwrap();
                        sigma1 += f64::powf((val1 as f64)-avg_1 as f64, 2.0);
                        sigma2 += f64::powf((val2 as f64)-avg_2 as f64, 2.0);

                    }
                }
                sigma1 = sigma1 / (lengthX*lengthY) as f64;
                sigma2 = sigma2 / (lengthX*lengthY) as f64;


                for vx in 0..lengthY+2 {
                    for ux in 0..lengthX+2 {
                        let mut y1 = yStart+vx+offsetY+v;
                        let mut x1 = xStart+ux+offsetX+ux;
                        if y1 < 0 {
                            y1 = 0;
                        }
                        if x1 < 0 {
                            x1 = 0;
                        }
                        let val1 = *image.at_2d::<u8>(y1, x1).unwrap() as f64;
                        let val2 = *image2.at_2d::<u8>(startY + vx , startX + ux).unwrap() as f64;

                        val += f64::abs(f64::powf( (val2 - avg_2 as f64) - (val1 - avg_1 as f64) , 2.0) / f64::sqrt(sigma1*sigma2));
                    }
                }
                val = val / (lengthX*lengthY) as f64;

                if val < SAD {
                    SAD = val;
                    saveOffsetX = -offsetX;
                    saveOffsetY = -offsetY;
                }
            }
            startX = startX+saveOffsetX;
            startY = startY+saveOffsetY;

            endX = endX+saveOffsetX;
            endY = endY+saveOffsetY;

            let newStart = Point::new(startX,startY);
            let newEnd = Point::new(endX,endY);

            imgproc::rectangle(&mut image2,
                               Rect::from_points(newStart,newEnd),  
                               color,
                               1,
                               imgproc::LINE_8,
                               0); 

        }

        img_array.push(image2.clone());
    }
    let mut vid = videoio::VideoWriter::new("tracking.avi",videoio::VideoWriter::fourcc('M','P','E','G').unwrap(), 15.0, size, false)?;

    for i in 0..img_array.len() {
        vid.write(&img_array[i])?;
    }
    vid.release()?;

    Ok(())
}
