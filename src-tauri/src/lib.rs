use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use csv::ReaderBuilder;
use std::io::BufReader;
use ndarray::Array1;
use std::fs;
use std::path::Path;

mod svg_utils;
pub mod spectrum;

// CSV files


/// Aproxima um espectro de LPFG por uma lorentziana em Rust
///
/// # Parâmetros
///
/// * `x` - Wavelength for simulation
/// * `a` - Attenuation intensity
/// * `x0` - Resonant wavelength
/// * `w` - FWHM
/// * `bias` - Insertion loss
///
/// # Retorna
///
/// * `spectrum` - LPFG array
pub fn transmission_spectra(x: Array1<f64>, a: f64, x0: f64, w: f64, bias: f64) -> Array1<f64> {
    let factor = w / (2.0 * (a / 3.0 - 1.0).abs().sqrt());
    x.mapv(|xi| -a * (1.0 + ((xi - x0) / factor).powi(2)).powf(-1.0) - bias)
}

pub fn my_gauss(x: Array1<f64>, a: f64, x0: f64, w: f64, bias: f64) -> Array1<f64> {
    let s = 2.0 * (4.0 * (a / 3.01).ln().abs()).sqrt();
    let s = w / s;
    let arg = x.mapv(|xi| -((xi - x0).powi(2) / (2.0 * s.powi(2))));
    arg.mapv(|arg_val| -a * arg_val.exp() - bias)
}

pub fn transmission_spectra_2(x: Array1<f64>, a: f64, x0: f64, w: f64, bias: f64, fcn: f64) -> Array1<f64> {
    if fcn < 0.5 {
        transmission_spectra(x, a, x0, w, bias)
    } else {
        let ts = transmission_spectra(x.clone(), a / 2.0, x0, w / 2.0, bias);
        let mg = my_gauss(x, a / 2.0, x0, w / 2.0, bias);
        ts + mg + bias
    }
}

// #[derive(Debug, Deserialize)]
// pub struct Record {
//     field1: String,
//     field2: String,
//     field3: i32,
// }

#[derive(Debug, Deserialize)]
pub struct Record {
    a: f64,
    x0: f64,
    w: f64,
    bias: f64,
}

pub fn read_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);
    
    let mut records = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    Ok(records)
}

pub fn read_txt(file_path: &str) -> Result<Vec<(f64, f64)>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_reader(reader);

    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let value1: f64 = record[0].parse()?;
        let value2: f64 = record[1].parse()?;
        records.push((value1, value2));
    }
    Ok(records)
}

pub fn show_data_txt(file_path: &str) {
    match read_txt(file_path) {
        Ok(records) => {
            for (i, (val1, val2)) in records.iter().enumerate() {
                println!("{}: {}, {}", i + 1, val1, val2);
            }
        }
        Err(err) => {
            eprintln!("Error reading file: {}", err);
        }
    }
}

pub fn show_strat(file_path: &str) {
    match read_csv(file_path) {
        Ok(records) => {
            for record in records {
                println!("{:?}", record);
            }
        }
        Err(err) => {
            eprintln!("Error reading CSV file: {}", err);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////

// fn import_TXT_Spectrum(file_path: &str) -> Result<Spectrum, Box<dyn Error>> {
//     // Converte o caminho do arquivo para um objeto Path
//     let path = Path::new(file_path);
    
//     // Lê o conteúdo do arquivo TXT para uma string
//     let txt_content = fs::read_to_string(path)?;
    
//     // Cria um objeto Spectrum a partir do texto do TXT
//     let spectrum = Spectrum::from_csv_text(&txt_content)?;
    
//     Ok(spectrum)
// }

////////////////////////////////////////////////////////////////////////////////////

fn select_file() -> String {
    // Abre o explorador de arquivos para selecionar um arquivo
    if let Some(file_path) = FileDialog::new().pick_file() {
        file_path.display().to_string()
    } else {
        "Nenhum arquivo foi selecionado.".to_string()
    }
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    let mut file = name.to_string();
    if name.len() < 5 {
        file = select_file();
        println!("Arquivo encontrado: {}", file);
        show_data_txt(&file);
        println!();
    } else {
        println!("ERRO: Não foi possível abrir o arquivo.");
    }
    format!("{}", file)
}

#[tauri::command]
fn return_file(name: &str) -> String {
    let mut file = name.to_string();
    if name.len() < 5 {
        file = select_file();
    }
    format!("File: {}", file)
}

#[tauri::command]
fn drag_file() -> String {
    println!("Drag event");
    format!("Drag event")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, return_file, drag_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
