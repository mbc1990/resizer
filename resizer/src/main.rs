extern crate inotify;
use threadpool::ThreadPool;

use std::env;
use std::path::{PathBuf, Path};

use inotify::{
    EventMask,
    WatchMask,
    Inotify,
};
use image::{GenericImageView, FilterType, DynamicImage};

const MAX_WIDTH: u32 = 800;
const IN_DIR: &str = "/home/malcolm/Downloads/cats2/fullsized/";
const OUT_DIR: &str = "/home/malcolm/Downloads/cats2/resized/";

fn main() {

    // Set up the worker pool
    let pool = ThreadPool::new(8);

    let mut inotify = Inotify::init()
        .expect("Failed to initialize inotify");

    let path = PathBuf::from(IN_DIR);

    inotify
        .add_watch(
            path,
            WatchMask::CREATE
        )
        .expect("Failed to add inotify watch");

    println!("Watching current directory for activity...");

    let mut buffer = [0u8; 4096];
    loop {
        let events = inotify
            .read_events_blocking(&mut buffer)
            .expect("Failed to read inotify events");

        for event in events {
            if event.mask.contains(EventMask::CREATE) {
                if event.mask.contains(EventMask::ISDIR) {
                    println!("Directory created: {:?}", event.name);
                } else {
                    // TODO: This works but seems unwieldy
                    let new_file_path = String::from(event.name.unwrap().to_str().unwrap());

                    // TODO: This can execute before the other writer finishes, which causes a panic
                    pool.execute(move|| {
                        let mut temp_in_path = IN_DIR.to_string();
                        temp_in_path.push_str(&new_file_path);
                        let in_path = Path::new(&temp_in_path);

                        let f_name = in_path.file_name().unwrap();
                        let mut temp_out_path = OUT_DIR.to_string();
                        temp_out_path.push_str(f_name.to_str().unwrap());
                        let out_path = Path::new(&temp_out_path);

                        println!("Trying to open path: {:?}", &in_path);

                        let img_res = image::open(&in_path);
                        match img_res {
                            Ok(img) => {
                                let (width, height) = img.dimensions();
                                let new_h = height as f32 * (MAX_WIDTH as f32 / width as f32);
                                let mut to_save: DynamicImage;
                                if width > MAX_WIDTH {
                                    println!("Resizing image...");
                                    to_save = img.resize(MAX_WIDTH, new_h as u32, FilterType::Lanczos3);
                                } else {
                                    to_save = img;
                                }
                                println!("Saving resized image to {:?}", &out_path);
                                to_save.save(out_path);
                            },
                            Err(e) => {
                                println!("{:?}", e);
                            }
                        }
                    });
                }
            }
        }
    }
}