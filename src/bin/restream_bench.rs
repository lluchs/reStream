use restream::compress::compress_buf_slow;

use std::time::Instant;
use std::fs::File;
use std::io::Read;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <image file>", args[0]);
        std::process::exit(1);
    }

    let mut buf = Vec::new();
    {
        let mut file = File::open(&args[1])?;
        file.read_to_end(&mut buf)?;
    }

    let mut compressed = Vec::with_capacity(buf.len() * 9 / 8);

    let start = Instant::now();
    let iterations = 100;
    if args[2] == "slow" {
        println!("SLOW");
        for _ in 0..iterations {
            compressed.clear();
            compress_buf_slow(&buf, &mut compressed);
        }
    } else if args[2] == "fast" {
        #[cfg(target_arch = "arm")]
        {
            use restream::compress::compress_buf_fast;
            println!("FAST");
            for _ in 0..iterations {
                compressed.clear();
                compress_buf_fast(&buf, &mut compressed);
            }
        }
    } else {
        println!("unknown speed");
    }

    let runtime = start.elapsed().as_secs_f64();
    println!("time: {:.3} secs", runtime);
    println!("bw:   {:.3} fps", iterations as f64 / runtime);
    println!("original:   {} bytes", buf.len());
    println!("compressed: {} bytes", compressed.len());
    println!("ratio: {}", buf.len() as f64 / compressed.len() as f64);

    Ok(())
}
