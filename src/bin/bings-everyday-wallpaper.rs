use native_dialog::*;
use once_cell::sync::OnceCell;
use std::fs;
use std::path::PathBuf;

static DIALOG: OnceCell<bool> = OnceCell::new();

fn panic_dialog(info: &std::panic::PanicInfo) {
    log::error!("use dialog to panic");

    let string = info.to_string();

    let result = MessageDialog::new()
        .set_title("Bings-everyday-wallpaper panic !")
        .set_text(&string)
        .set_type(MessageType::Error)
        .show_confirm();

    if let Err(err) = result {
        println!("Dialog error:\n{:?}", err);
    }
}

#[derive(Debug, Default, Clone)]
struct Args {
    image_path: PathBuf,
}

const IMAGE_DEFAULT_NAME: &str = "bings-everyday-wallpaper.jpg";

fn parse_arg() -> eyre::Result<Args> {
    use clap::*;

    let matches = Command::new("bings-everyday-wallpaper")
        .arg(Arg::new("path").required(true))
        .arg(
            Arg::new("dialog")
                .short('d')
                .long("dialog")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let dialog = matches.get_flag("dialog");
    DIALOG.set(dialog).unwrap();
    if dialog {
        std::panic::set_hook(Box::new(panic_dialog));
    }

    let path = matches.get_one::<String>("path").unwrap().clone();
    let mut path = PathBuf::from(path);
    process_path(&mut path)?;

    Ok(crate::Args { image_path: path })
}

fn process_path(path: &mut PathBuf) -> eyre::Result<()> {
    if path.exists() {
        if path.is_dir() {
            path.push(IMAGE_DEFAULT_NAME);
        }
    } else {
        if path.extension().is_none() {
            fs::create_dir_all(path)?;
        }
    }
    Ok(())
}

async fn download(arg: &Args) -> eyre::Result<()> {
    // require for image url
    let url = "https://cn.bing.com/HPImageArchive.aspx?format=js&n=1";
    let json: serde_json::Value = reqwest::get(url).await?.json().await?;
    let image_url = json["images"][0]["url"].as_str().unwrap();
    let image_url = format!("https://www.bing.com{}", image_url);

    // download it
    let bytes = reqwest::get(image_url).await?.bytes().await?;
    fs::write(&arg.image_path, bytes)?;
    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();

    let arg = parse_arg()?;

    println!("Downloading ...");

    download(&arg).await?;

    println!(
        "Download completed , the image will save to \'{}\'",
        arg.image_path.display()
    );

    Ok(())
}
