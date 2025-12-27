#![allow(unused_imports)] 

use std::io::{Read, Write, BufReader, BufWriter};
use std::net::{TcpStream, TcpListener};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use chrono::DateTime;

const REAL_HOST: &str = "95.163.237.76";
const PORT_1: u16 = 5123;
const PORT_2: u16 = 5124;
const AUTH_KEY: &str = "isu_pt";
const CMD_GET: &str = "get";
const OUTPUT_FILE: &str = "sensor_data.txt";
const REQUEST_DELAY: Duration = Duration::from_millis(25);

fn verify_checksum(buffer: &[u8]) -> bool {
    if buffer.is_empty() { return false; }
    let len = buffer.len() - 1;
    let expected = buffer[len];
    let mut sum: u32 = 0;
    for &b in &buffer[..len] {
        sum = sum.wrapping_add(b as u32);
    }
    (sum % 256) as u8 == expected
}

fn format_time(micro_ts: i64) -> String {
    let secs = micro_ts / 1_000_000;
    let nsecs = (micro_ts % 1_000_000) * 1_000;
    match DateTime::from_timestamp(secs, nsecs as u32) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => "INVALID_TIME".to_string(),
    }
}

fn parse_port_5123(buffer: &[u8]) -> String {
    let time = i64::from_be_bytes(buffer[0..8].try_into().unwrap());
    let temp = f32::from_be_bytes(buffer[8..12].try_into().unwrap());
    let press = i16::from_be_bytes(buffer[12..14].try_into().unwrap());
    format!("{} | 5123 | Temp: {:.2} | Press: {}", format_time(time), temp, press)
}

fn parse_port_5124(buffer: &[u8]) -> String {
    let time = i64::from_be_bytes(buffer[0..8].try_into().unwrap());
    let x = i32::from_be_bytes(buffer[8..12].try_into().unwrap());
    let y = i32::from_be_bytes(buffer[12..16].try_into().unwrap());
    let z = i32::from_be_bytes(buffer[16..20].try_into().unwrap());
    format!("{} | 5124 | X: {} | Y: {} | Z: {}", format_time(time), x, y, z)
}

fn run_client(
    host: &str,
    port: u16,
    packet_size: usize,
    tx: Sender<String>,
    is_running: Arc<AtomicBool>,
) {
    let addr = format!("{}:{}", host, port);
    let mut data_buf = vec![0u8; packet_size];
    let mut greet_buf = vec![0u8; 7]; 

    while is_running.load(Ordering::Relaxed) {
        match TcpStream::connect(&addr) {
            Ok(stream) => {
                let _ = stream.set_nodelay(true); 
                let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
                let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
                let stream_clone = stream.try_clone().expect("Clone failed");
                let mut reader = BufReader::new(stream);
                let mut writer = BufWriter::new(stream_clone);

                if writer.write_all(AUTH_KEY.as_bytes()).is_err() || writer.flush().is_err() { 
                    continue; 
                }
                thread::sleep(Duration::from_millis(500));

                if reader.read_exact(&mut greet_buf).is_err() {
                    eprintln!("[Port {}] Handshake failed. Reconnecting...", port);
                    continue;
                }
                println!("[Port {}] Connected.", port);

                while is_running.load(Ordering::Relaxed) {
                    if writer.write_all(CMD_GET.as_bytes()).is_err() || writer.flush().is_err() { 
                        eprintln!("[Port {}] Write error. Reconnecting...", port);
                        break; 
                    }
                    match reader.read_exact(&mut data_buf) {
                        Ok(_) => {
                            if verify_checksum(&data_buf) {
                                let msg = if port == 5123 { 
                                    parse_port_5123(&data_buf) 
                                } else { 
                                    parse_port_5124(&data_buf) 
                                };
                                if tx.send(msg).is_err() { break; }
                            }
                        }
                        Err(_) => {
                            eprintln!("[Port {}] Disconnected (EOF). Reconnecting...", port);
                            break;
                        }
                    }
                    thread::sleep(REQUEST_DELAY);
                }
            }
            Err(_) => {
                if host != "127.0.0.1" { thread::sleep(Duration::from_secs(2)); }
            }
        }
    }
}

fn run_writer_loop(
    mut writer: Box<dyn Write>,
    rx: Receiver<String>,
    is_running: Arc<AtomicBool>
) -> u64 {
    let mut count: u64 = 0;
    let start_time = Instant::now();
    let mut last_log = Instant::now();

    loop {
        if !is_running.load(Ordering::Relaxed) {
             for msg in rx.try_iter() {
                 let _ = writeln!(writer, "{}", msg);
             }
             break;
        }
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(msg) => {
                let _ = writeln!(writer, "{}", msg);
                count += 1;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if last_log.elapsed() >= Duration::from_secs(5) {
            let total = start_time.elapsed().as_secs();
            println!("Running: {:02}:{:02} | Total lines: {}", total / 60, total % 60, count);
            let _ = writer.flush();
            last_log = Instant::now();
        }
    }
    let _ = writer.flush();
    count
}

#[cfg(not(test))]
fn main() {
    let is_running = Arc::new(AtomicBool::new(true));
    let r_sig = is_running.clone();

    ctrlc::set_handler(move || {
        println!("\nStopping...");
        r_sig.store(false, Ordering::Relaxed);
    }).expect("Error setting Ctrl-C");

    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    println!("Target: {}", REAL_HOST);
    println!("File: {}", OUTPUT_FILE);

    let tx1 = tx.clone();
    let r1 = is_running.clone();
    thread::spawn(move || run_client(REAL_HOST, PORT_1, 15, tx1, r1));

    let tx2 = tx.clone();
    let r2 = is_running.clone();
    thread::spawn(move || run_client(REAL_HOST, PORT_2, 21, tx2, r2));

    let file = std::fs::File::create(OUTPUT_FILE).expect("Create file failed");
    let writer = Box::new(BufWriter::new(file));
    
    let count = run_writer_loop(writer, rx, is_running);

    println!("\nDone. Total: {}", count);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Проверяет, что сумма байт по модулю 256 совпадает с последним байтом
    #[test]
    fn test_verify_checksum() {
        // Пустой буфер - false
        assert_eq!(verify_checksum(&[]), false);
        // Корректно: 1 + 2 = 3
        assert_eq!(verify_checksum(&[1, 2, 3]), true);
        // Некорректно: 1 + 2 != 4
        assert_eq!(verify_checksum(&[1, 2, 4]), false);
        // Переполнение: 250 + 10 = 260 -> 260 % 256 = 4
        assert_eq!(verify_checksum(&[250, 10, 4]), true);
    }

    // Проверяет конвертацию UNIX timestamp микросекунды в строку
    #[test]
    fn test_format_time() {
        // 0 -> Epoch start
        assert_eq!(format_time(0), "1970-01-01 00:00:00");
    }

    // Тест парсинга данных Порт 5123
    #[test]
    fn test_parse_5123() {
        let mut buf = vec![0u8; 15];
        buf[8] = 0x3F; buf[9] = 0x80; 
        buf[13] = 0x0A;
        let res = parse_port_5123(&buf);
        assert!(res.contains("5123"));
        assert!(res.contains("Temp: 1.00"));
        assert!(res.contains("Press: 10"));
    }

    // Тест парсинга данных Порт 5124
    #[test]
    fn test_parse_5124() {
        let mut buf = vec![0u8; 21];
        buf[11] = 1;
        let res = parse_port_5124(&buf);
        assert!(res.contains("5124"));
        assert!(res.contains("X: 1"));
    }

    // Проверяет, что данные из канала корректно записываются в файл
    #[test]
    fn test_writer_loop() {
        let is_running = Arc::new(AtomicBool::new(true));
        let (tx, rx) = mpsc::channel();
        // Посылаем данные
        tx.send("Test Line 1".to_string()).unwrap();
        // Запускаем остановку в отдельном потоке
        let r_clone = is_running.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            r_clone.store(false, Ordering::Relaxed);
        });
        // Пишем в память вместо файла
        let buffer = Vec::new();
        let writer = Box::new(Cursor::new(buffer));
        let count = run_writer_loop(writer, rx, is_running);
        assert_eq!(count, 1);
    }

    // Поднимает локальный TCP-сервер, эмулирует протокол и проверяет,
    // что клиент успешно проходит авторизацию и получает данные
    #[test]
    fn test_run_client_integration() {
        // Поднимаем сервер на случайном порту
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_clone = is_running.clone();
        let (tx, rx) = mpsc::channel();
        // Запускаем клиента в потоке
        let handle = thread::spawn(move || {
            // packet_size 21 соответствует ветке else (не 5123)
            run_client("127.0.0.1", port, 21, tx, is_running_clone);
        });
        // Эмуляция сервера
        let (mut stream, _) = listener.accept().unwrap();
        // A. Сервер читает ключ "isu_pt"
        let mut key_buf = [0u8; 6];
        stream.read_exact(&mut key_buf).unwrap();
        assert_eq!(&key_buf, b"isu_pt");
        // B. Сервер отправляет приветствие (7 байт)
        stream.write_all(&[0u8; 7]).unwrap();
        // C. Сервер читает команду "get"
        let mut get_buf = [0u8; 3];
        stream.read_exact(&mut get_buf).unwrap();
        assert_eq!(&get_buf, b"get");
        // D. Сервер отправляет пакет данных (21 байт)
        // Все нули -> CRC (последний байт) тоже 0 -> валидный пакет
        stream.write_all(&[0u8; 21]).unwrap();
        // Останавливаем тест
        thread::sleep(Duration::from_millis(100));
        is_running.store(false, Ordering::Relaxed);
        handle.join().unwrap();
        // Проверяем, что клиент отправил распарсенную строку в канал
        let msg = rx.recv().unwrap();
        assert!(msg.contains("5124"));
    }

    // Проверяет, что клиент не падает panic, если порта не существует
    #[test]
    fn test_connection_refused_no_panic() {
        let is_running = Arc::new(AtomicBool::new(true));
        let (tx, _rx) = mpsc::channel();
        let r_clone = is_running.clone();
        // Таймер остановки
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            r_clone.store(false, Ordering::Relaxed);
        });
        // Пытаемся подключиться к заведомо закрытому порту
        run_client("127.0.0.1", 55555, 15, tx, is_running);
    }
}