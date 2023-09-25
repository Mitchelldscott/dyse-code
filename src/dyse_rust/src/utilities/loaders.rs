/********************************************************************************
 *
 *      ____                     ____          __           __       _
 *     / __ \__  __________     /  _/___  ____/ /_  _______/ /______(_)__  _____
 *    / / / / / / / ___/ _ \    / // __ \/ __  / / / / ___/ __/ ___/ / _ \/ ___/
 *   / /_/ / /_/ (__  )  __/  _/ // / / / /_/ / /_/ (__  ) /_/ /  / /  __(__  )
 *  /_____/\__, /____/\___/  /___/_/ /_/\__,_/\__,_/____/\__/_/  /_/\___/____/
 *        /____/
 *
 *
 *
 ********************************************************************************/

extern crate yaml_rust;

// use crate::comms::data_structures::*;
use glob::glob;
use std::{env, fmt, fs, path};
use yaml_rust::{yaml::Yaml, YamlLoader};

pub static MAX_RECORDS_PER_CSV: u16 = 10000;
pub static MAX_FILES_PER_RUN: u16 = 120;

pub fn assert_path(path: &path::Path) {
    if !path.exists() {
        fs::create_dir_all(path).unwrap();
    }
}

pub fn assert_file(path: &path::Path) {
    if !path.exists() {
        fs::File::create(path).unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct ByuParseError {
    pub item: String,
    pub yaml_file: String,
}

impl ByuParseError {
    pub fn new(item: String, yaml_file: &str) -> ByuParseError {
        ByuParseError {
            item: item,
            yaml_file: yaml_file.to_string(),
        }
    }

    pub fn int(item: &str, yaml_file: &str) -> ByuParseError {
        ByuParseError::new(format!("{item}: i64"), yaml_file)
    }

    pub fn float(item: &str, yaml_file: &str) -> ByuParseError {
        ByuParseError::new(format!("{item}: f64"), yaml_file)
    }

    pub fn string(item: &str, yaml_file: &str) -> ByuParseError {
        ByuParseError::new(format!("{item}: String"), yaml_file)
    }

    pub fn item(item: &str, yaml_file: &str) -> ByuParseError {
        ByuParseError::new(format!("{item}: Item"), yaml_file)
    }
}

impl fmt::Display for ByuParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} not found in {}", self.item, self.yaml_file)
    }
}

pub struct BuffYamlUtil {
    pub yaml_path: String,
    pub data: Yaml,
}

impl BuffYamlUtil {
    pub fn read_yaml_as_string(yaml_path: &str) -> String {
        fs::read_to_string(yaml_path).expect(format!("No config in {}", yaml_path).as_str())
    }

    pub fn new(data: &str) -> BuffYamlUtil {
        BuffYamlUtil {
            yaml_path: "no file".to_string(),
            data: YamlLoader::load_from_str(data).unwrap()[0].clone(),
        }
    }

    pub fn robot(name: &str, yaml_name: &str) -> BuffYamlUtil {
        let project_root = env::var("PROJECT_ROOT").expect("Project root not set");
        let yaml_path = format!("{}/dysepy/data/robots/{}", project_root, name);

        let data_string = BuffYamlUtil::read_yaml_as_string(
            format!("{}/{}.yaml", yaml_path, yaml_name,).as_str(),
        );

        BuffYamlUtil {
            yaml_path: yaml_path,
            data: YamlLoader::load_from_str(data_string.as_str()).unwrap()[0].clone(),
        }
    }

    pub fn from_self(yaml_name: &str) -> BuffYamlUtil {
        let project_root = env::var("PROJECT_ROOT").expect("Project root not set");
        BuffYamlUtil::robot(
            fs::read_to_string(format!("{}/dysepy/data/robots/self.txt", project_root))
                .unwrap()
                .as_str(),
            yaml_name,
        )
    }

    pub fn default(yaml_name: &str) -> BuffYamlUtil {
        match env::var("ROBOT_NAME") {
            Ok(robot_name) => BuffYamlUtil::robot(&robot_name.as_str(), yaml_name),
            _ => BuffYamlUtil::from_self(yaml_name),
        }
    }

    pub fn data(&self) -> &Yaml {
        &self.data
    }

    pub fn item(&self, item: &str) -> Result<&Yaml, ByuParseError> {
        match self.data[item].is_badvalue() {
            false => Ok(&self.data[item]),
            true => Err(ByuParseError::item(item, &self.yaml_path)),
        }
    }

    pub fn parse_int(&self, item: &str, data: &Yaml) -> Result<i64, ByuParseError> {
        match &data[item] {
            Yaml::Integer(val) => Ok(*val),
            _ => Err(ByuParseError::int(item, &self.yaml_path)),
        }
    }

    pub fn parse_float(&self, item: &str, data: &Yaml) -> Result<f64, ByuParseError> {
        match &data[item] {
            Yaml::Real(val) => Ok(val.parse::<f64>().unwrap()),
            _ => Err(ByuParseError::float(item, &self.yaml_path)),
        }
    }

    pub fn parse_str(&self, item: &str, data: &Yaml) -> Result<String, ByuParseError> {
        match &data[item] {
            Yaml::String(val) => Ok(val.clone()),
            _ => Err(ByuParseError::string(item, &self.yaml_path)),
        }
    }

    pub fn parse_ints(&self, item: &str, data: &Yaml) -> Result<Vec<i64>, ByuParseError> {
        match &data[item] {
            Yaml::Array(list) => list
                .iter()
                .map(|x| match x {
                    Yaml::Integer(val) => Ok(*val),
                    _ => Err(ByuParseError::int(item, &self.yaml_path)),
                })
                .collect(),
            _ => Err(ByuParseError::int(item, &self.yaml_path)),
        }
    }

    pub fn parse_floats(&self, item: &str, data: &Yaml) -> Result<Vec<f64>, ByuParseError> {
        match &data[item] {
            Yaml::Array(list) => list
                .iter()
                .map(|x| match x {
                    Yaml::Real(val) => Ok(val.parse::<f64>().unwrap()),
                    _ => Err(ByuParseError::float(item, &self.yaml_path)),
                })
                .collect(),
            _ => Err(ByuParseError::int(item, &self.yaml_path)),
        }
    }

    pub fn parse_strs(&self, item: &str, data: &Yaml) -> Result<Vec<String>, ByuParseError> {
        match &data[item] {
            Yaml::Array(list) => list
                .iter()
                .map(|x| match x {
                    Yaml::String(val) => Ok(val.clone()),
                    _ => Err(ByuParseError::int(item, &self.yaml_path)),
                })
                .collect(),
            _ => Err(ByuParseError::int(item, &self.yaml_path)),
        }
    }
}

// #[derive(Clone)]
pub struct CsvUtil {
    pub run: u16,
    pub name: String,
    pub file_count: u16,
    pub record_count: u16,
    pub labels: Vec<String>,
    writer: csv::Writer<fs::File>,
    reader: csv::Reader<fs::File>,
}

impl CsvUtil {
    pub fn make_run_directory(name: &str, run: u16) -> String {
        let robot_name = env::var("ROBOT_NAME").expect("Robot name not set");
        let project_root = env::var("PROJECT_ROOT").expect("Project root not set");
        let path = format!("{project_root}/data/{robot_name}/{name}/{run}");
        assert_path(&path::Path::new(&path));
        path
    }

    pub fn make_path(name: &str, run: u16, file_count: u16) -> String {
        let ppath = CsvUtil::make_run_directory(name, run);
        let path = format!("{}/{}.csv", ppath, file_count);

        assert_file(&path::Path::new(&path));
        path
    }

    pub fn new(name: &str, labels: Vec<String>) -> CsvUtil {
        let path = CsvUtil::make_path(name, 0, 0);
        CsvUtil {
            run: 0,
            name: name.to_string(),
            file_count: 0,
            record_count: 0, // always start with a fresh run
            labels: labels,
            writer: csv::Writer::from_path(path.clone()).unwrap(),
            reader: csv::Reader::from_path(path).unwrap(),
        }
    }

    pub fn new_labels(&mut self, labels: Vec<String>) {
        self.labels = labels;
    }

    pub fn set_run(&mut self, run: u16, fc: u16) {
        self.run = run;
        self.file_count = fc;
        self.new_reader();
        self.new_writer();
    }

    pub fn new_writer(&mut self) {
        let path = CsvUtil::make_path(&self.name, self.run, self.file_count);
        self.writer = csv::Writer::from_path(path).unwrap()
    }

    pub fn new_reader(&mut self) {
        let path = CsvUtil::make_path(&self.name, self.run, self.file_count);
        self.reader = csv::Reader::from_path(path).unwrap()
    }

    pub fn new_run(&mut self) {
        let robot_name = env::var("ROBOT_NAME").expect("Robot name not set");
        let project_root = env::var("PROJECT_ROOT").expect("Project root not set");

        let data_path = format!("{project_root}/data/{robot_name}/{}/*", self.name);
        self.run = glob(&data_path).unwrap().count() as u16;
    }

    pub fn write_record(&mut self, record: Vec<f64>) {
        if self.labels.len() > 1 {
            if self.record_count == 0 {
                self.writer.write_record(&self.labels).unwrap();
            }

            self.writer
                .write_record(
                    record
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>(),
                )
                .expect("[CsvUtil]: Failed to write");

            self.record_count += 1;
        }

        if self.record_count > MAX_RECORDS_PER_CSV {
            self.writer.flush().unwrap();
            self.record_count = 0;
            self.file_count += 1;

            if self.file_count > MAX_FILES_PER_RUN {
                self.file_count = 0;
                self.new_run();
            }

            self.new_writer();
        }
    }

    pub fn save(&mut self, data: &Vec<f64>, data_timestamp: f64) {
        if data.len() > 0 {
            self.write_record(data.iter().map(|x| *x).chain([data_timestamp]).collect());
        }
    }

    pub fn load_file(&mut self) -> Vec<Vec<f64>> {
        self.reader
            .records()
            .map(|record| record.unwrap().iter().map(|x| x.parse().unwrap()).collect())
            .collect()
    }

    pub fn load_run(&mut self, run: u16, fc: u16) -> Vec<Vec<f64>> {
        let robot_name = env::var("ROBOT_NAME").expect("Robot name not set");
        let project_root = env::var("PROJECT_ROOT").expect("Project root not set");

        let data_path = format!("{project_root}/data/{robot_name}/{}/{run}/*", self.name);

        self.run = run;

        (0..std::cmp::min(glob(&data_path).unwrap().count() as u16, fc))
            .map(|fc| {
                self.file_count = fc;
                self.new_reader();
                self.load_file()
            })
            .flatten()
            .collect()
    }
}
