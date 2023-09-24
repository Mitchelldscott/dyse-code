extern crate yaml_rust;

use crate::comms::data_structures::*;
use glob::glob;
use std::{env, fs, path};
use yaml_rust::{yaml::Yaml, Yaml::Integer, YamlLoader};

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

pub struct BuffYamlUtil {
    pub yaml_path: String,
    pub node_data: Yaml,
    pub task_data: Yaml,
}

impl BuffYamlUtil {
    pub fn read_yaml_as_string(yaml_path: &str) -> String {
        fs::read_to_string(yaml_path).expect(format!("No config in {}", yaml_path).as_str())
    }

    pub fn new(bot_name: &str) -> BuffYamlUtil {
        let robot_name = bot_name;
        let project_root = env::var("PROJECT_ROOT").expect("Project root not set");

        let yaml_path = format!("{}/dysepy/data/robots/{}", project_root, robot_name);

        let node_string = BuffYamlUtil::read_yaml_as_string(
            format!(
                "{}/dysepy/data/robots/{}/nodes.yaml",
                project_root, robot_name
            )
            .as_str(),
        );

        let task_string = BuffYamlUtil::read_yaml_as_string(
            format!(
                "{}/dysepy/data/robots/{}/firmware_tasks.yaml",
                project_root, robot_name
            )
            .as_str(),
        );

        BuffYamlUtil {
            yaml_path: yaml_path,
            node_data: YamlLoader::load_from_str(node_string.as_str()).unwrap()[0].clone(),
            task_data: YamlLoader::load_from_str(task_string.as_str()).unwrap()[0].clone(),
        }
    }

    pub fn from_self() -> BuffYamlUtil {
        let project_root = env::var("PROJECT_ROOT").expect("Project root not set");
        let self_path = format!("{}/dysepy/data/robots/self.txt", project_root);
        let robot_name = fs::read_to_string(self_path).unwrap();
        BuffYamlUtil::new(robot_name.as_str())
    }

    pub fn default() -> BuffYamlUtil {
        match env::var("ROBOT_NAME") {
            Ok(robot_name) => BuffYamlUtil::new(&robot_name.as_str()),
            _ => BuffYamlUtil::from_self(),
        }
    }

    pub fn load_string(&self, item: &str) -> String {
        self.node_data[item].as_str().unwrap().to_string()
    }

    pub fn load_u16(&self, item: &str) -> u16 {
        self.node_data[item].as_i64().unwrap() as u16
    }

    pub fn load_u128(&self, item: &str) -> u128 {
        self.node_data[item].as_i64().unwrap() as u128
    }

    pub fn load_string_list(&self, item: &str) -> Vec<String> {
        self.node_data[item]
            .as_vec()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().to_string())
            .collect()
    }

    pub fn load_u8_list(&self, item: &str) -> Vec<u8> {
        self.node_data[item]
            .as_vec()
            .unwrap()
            .iter()
            .map(|x| x.as_i64().unwrap() as u8)
            .collect()
    }

    pub fn load_integer_matrix(&self, item: &str) -> Vec<Vec<u8>> {
        self.node_data[item]
            .as_vec()
            .unwrap()
            .iter()
            .map(|x| {
                x.as_vec()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_i64().unwrap() as u8)
                    .collect()
            })
            .collect()
    }

    pub fn load_float_matrix(&self, item: &str) -> Vec<Vec<f64>> {
        self.node_data[item]
            .as_vec()
            .unwrap()
            .iter()
            .map(|x| {
                x.as_vec()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_f64().unwrap())
                    .collect()
            })
            .collect()
    }

    // clean up at some point, use a function to search a string in the hash of an item or something
    pub fn parse_tasks(data: &Yaml) -> Vec<EmbeddedTask> {
        data.as_hash()
            .unwrap()
            .iter()
            .map(|(key, value)| {
                let name = key.as_str().unwrap();
                let mut inputs = vec![];
                let mut config = vec![];
                let mut driver = "UNKNOWN".to_string();
                let mut rate = 250;
                let mut record = 0;
                let mut publish = 0;

                value.as_vec().unwrap().iter().for_each(|item| {
                    item.as_hash()
                        .unwrap()
                        .iter()
                        .for_each(|(k, v)| match k.as_str().unwrap() {
                            "inputs" => {
                                inputs = v
                                    .as_vec()
                                    .unwrap()
                                    .to_vec()
                                    .iter()
                                    .map(|name| name.as_str().unwrap().to_string())
                                    .collect();
                            }
                            "parameters" => {
                                v.as_vec().unwrap().iter().for_each(|x| match x {
                                    Yaml::Real(_) => config.push(x.as_f64().unwrap()),
                                    Yaml::Array(a) => {
                                        a.iter().for_each(|n| config.push(n.as_f64().unwrap()))
                                    }
                                    _ => {}
                                });
                            }
                            "driver" => {
                                driver = v.as_str().unwrap().to_string();
                            }
                            "rate" => {
                                rate = match v {
                                    Integer(val) => *val as u16,
                                    _ => 0,
                                };
                            }
                            "record" => {
                                record = match v {
                                    Integer(val) => *val as u16,
                                    _ => 0,
                                };
                            }
                            "publish" => {
                                publish = match v {
                                    Integer(val) => *val as u16,
                                    _ => 0,
                                };
                            }
                            _ => {}
                        });
                });
                EmbeddedTask::named(
                    name.to_string(),
                    driver,
                    inputs,
                    config,
                    rate,
                    record,
                    publish,
                )
            })
            .collect()
    }

    pub fn load_tasks(&self) -> Vec<EmbeddedTask> {
        BuffYamlUtil::parse_tasks(&self.task_data)
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
