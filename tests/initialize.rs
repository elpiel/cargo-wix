// Copyright (C) 2017 Christopher R. Field.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate assert_fs;
extern crate cargo_wix;
#[macro_use] extern crate lazy_static;
extern crate predicates;
extern crate tempfile;
extern crate toml;

mod common;

use assert_fs::prelude::*;
use predicates::prelude::*;

use cargo_wix::{LICENSE_FILE_NAME, RTF_FILE_EXTENSION, WIX_SOURCE_FILE_NAME, WIX_SOURCE_FILE_EXTENSION, WIX};
use cargo_wix::initialize::{Builder, Execution};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use toml::Value;

lazy_static!{
    static ref MAIN_WXS: String = WIX_SOURCE_FILE_NAME.to_owned() + "." + WIX_SOURCE_FILE_EXTENSION;
    static ref LICENSE_RTF: String = LICENSE_FILE_NAME.to_owned() + "." + RTF_FILE_EXTENSION;
    static ref WIX_PATH: PathBuf = PathBuf::from(WIX);
    static ref MAIN_WXS_PATH: PathBuf = PathBuf::from(WIX).join(MAIN_WXS.as_str());
    static ref LICENSE_RTF_PATH: PathBuf = PathBuf::from(WIX).join(LICENSE_RTF.as_str());
}

#[test]
fn default_works() {
    // Save the current working directory so that we can change back to it at
    // the end of the test. This avoids polluting the `tests` folder for the
    // source code with test artifacts.
    let original_working_directory = env::current_dir().unwrap();
    let package = common::create_test_package();
    env::set_current_dir(package.path()).unwrap();
    let result = Execution::default().run();
    env::set_current_dir(original_working_directory).unwrap();
    assert!(result.is_ok());
    package.child(WIX_PATH.as_path()).assert(predicate::path::exists());
    package.child(MAIN_WXS_PATH.as_path()).assert(predicate::path::exists());
    package.child(LICENSE_RTF_PATH.as_path()).assert(predicate::path::missing());
}

#[test]
fn description_works() {
    const EXPECTED: &str = "This is a description";
    let original_working_directory = env::current_dir().unwrap();
    let package = common::create_test_package();
    env::set_current_dir(package.path()).unwrap();
    let result = Builder::default().description(Some(EXPECTED)).build().run();
    env::set_current_dir(original_working_directory).unwrap();
    assert!(result.is_ok());
    let actual = common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "/wix:Wix/wix:Product/wix:Package/@Description"
    );
    assert_eq!(actual, EXPECTED);
}

#[test]
fn help_url_works() {
    const EXPECTED: &str = "http://www.example.com";
    let original_working_directory = env::current_dir().unwrap();
    let package = common::create_test_package();
    env::set_current_dir(package.path()).unwrap();
    let result = Builder::default().help_url(Some(EXPECTED)).build().run();
    env::set_current_dir(original_working_directory).unwrap();
    assert!(result.is_ok());
    let actual = common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "/wix:Wix/wix:Product/wix:Property[@Id='ARPHELPLINK']/@Value"
    );
    assert_eq!(actual, EXPECTED);
}

#[test]
fn manufacturer_works() {
    const EXPECTED: &str = "Example Manufacturer";
    let original_working_directory = env::current_dir().unwrap();
    let package = common::create_test_package();
    env::set_current_dir(package.path()).unwrap();
    let result = Builder::default().manufacturer(Some(EXPECTED)).build().run();
    env::set_current_dir(original_working_directory).unwrap();
    assert!(result.is_ok());
    let actual = common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "/wix:Wix/wix:Product/wix:Package/@Manufacturer"
    );
    assert_eq!(actual, EXPECTED);
}

#[test]
fn product_name_works() {
    const EXPECTED: &str = "Example Product Name";
    let original_working_directory = env::current_dir().unwrap();
    let package = common::create_test_package();
    env::set_current_dir(package.path()).unwrap();
    let result = Builder::default().product_name(Some(EXPECTED)).build().run();
    env::set_current_dir(original_working_directory).unwrap();
    assert!(result.is_ok());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "/wix:Wix/wix:Product/@Name"
    ), EXPECTED);
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "/wix:Wix/wix:Product/wix:Property[@Id='DiskPrompt']/@Value"
    ), EXPECTED.to_string() + " Installation");
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:Directory[@Id='APPLICATIONFOLDER']/@Name"
    ), EXPECTED);
}

#[test]
fn binary_name_works() {
    const EXPECTED: &str = "Example";
    let original_working_directory = env::current_dir().unwrap();
    let package = common::create_test_package();
    env::set_current_dir(package.path()).unwrap();
    let result = Builder::default().binary_name(Some(EXPECTED)).build().run();
    env::set_current_dir(original_working_directory).unwrap();
    assert!(result.is_ok());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        &format!("//*/wix:File[@Id='{}EXE']/@Name", EXPECTED)
    ), EXPECTED.to_string() + ".exe");
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        &format!("//*/wix:File[@Id='{}EXE']/@Source", EXPECTED)
    ), format!("target\\release\\{}.exe", EXPECTED));
}

#[test]
fn input_works() {
    let package = common::create_test_package();
    let result = Builder::default()
        .input(package.child("Cargo.toml").path().to_str())
        .build()
        .run();
    assert!(result.is_ok());
    package.child(WIX).assert(predicate::path::exists());
    package.child(MAIN_WXS_PATH.as_path()).assert(predicate::path::exists());
    package.child(LICENSE_RTF_PATH.as_path()).assert(predicate::path::missing());
}

#[test]
fn output_works() {
    let original_working_directory = env::current_dir().unwrap();
    let package = common::create_test_package();
    let output = tempfile::Builder::new().prefix("cargo_wix_test_output_").tempdir().unwrap();
    env::set_current_dir(package.path()).unwrap();
    let result = Builder::default()
        .output(output.path().to_str())
        .build()
        .run();
    env::set_current_dir(original_working_directory).unwrap();
    assert!(result.is_ok());
    output.child(MAIN_WXS.as_str()).assert(predicate::path::exists());
}

#[test]
fn license_with_txt_file_works() {
    const EXPECTED: &str = "License_Example.txt";
    let original_working_directory = env::current_dir().unwrap();
    let package = common::create_test_package();
    let package_license = package.child(EXPECTED);
    env::set_current_dir(package.path()).unwrap();
    let _license_handle = File::create(package_license.path()).unwrap();
    let result = Builder::default()
        .license(package_license.path().to_str())
        .build()
        .run();
    env::set_current_dir(original_working_directory).unwrap();
    assert!(result.is_ok());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:File[@Id='LicenseFile']/@Name"
    ), EXPECTED);
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:File[@Id='LicenseFile']/@Source"
    ), package_license.path().to_str().unwrap());
}

#[test]
fn license_with_rtf_file_works() {
    const EXPECTED: &str = "License_Example.rtf";
    let original_working_directory = env::current_dir().unwrap();
    let package = common::create_test_package();
    let package_license = package.child(EXPECTED);
    env::set_current_dir(package.path()).unwrap();
    let _license_handle = File::create(package_license.path()).unwrap();
    let result = Builder::default()
        .license(package_license.path().to_str())
        .build()
        .run();
    env::set_current_dir(original_working_directory).unwrap();
    assert!(result.is_ok());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:File[@Id='LicenseFile']/@Name"
    ), EXPECTED);
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:File[@Id='LicenseFile']/@Source"
    ), package_license.path().to_str().unwrap());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:WixVariable[@Id='WixUILicenseRtf']/@Value"
    ), package_license.path().to_str().unwrap());
}

#[test]
fn eula_works() {
    const EXPECTED: &str = "EULA_Example.rtf";
    let original_working_directory = env::current_dir().unwrap();
    let package = common::create_test_package();
    let package_eula = package.child(EXPECTED);
    env::set_current_dir(package.path()).unwrap();
    let _eula_handle = File::create(package_eula.path()).unwrap();
    let result = Builder::default()
        .eula(package_eula.path().to_str())
        .build()
        .run();
    env::set_current_dir(original_working_directory).unwrap();
    assert!(result.is_ok());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:WixVariable[@Id='WixUILicenseRtf']/@Value"
    ), package_eula.path().to_str().unwrap());
}

#[test]
fn mit_license_id_works() {
    let original_working_directory = env::current_dir().unwrap();
    let package = common::create_test_package();
    env::set_current_dir(package.path()).unwrap();
    let package_manifest = package.child("Cargo.toml");
    let mut toml: Value = {
        let mut cargo_toml_handle = File::open(package_manifest.path()).unwrap();
        let mut cargo_toml_content = String::new();
        cargo_toml_handle.read_to_string(&mut cargo_toml_content).unwrap();
        toml::from_str(&cargo_toml_content).unwrap()
    };
    {
        toml.get_mut("package").and_then(|p| {
            match p {
                Value::Table(ref mut t) => t.insert(String::from("license"), Value::from("MIT")),
                _ => panic!("The 'package' section is not a table"),
            };
            Some(p)
        }).expect("A package section for the Cargo.toml");
        let toml_string = toml.to_string();
        let mut cargo_toml_handle = File::create(package_manifest.path()).unwrap();
        cargo_toml_handle.write_all(toml_string.as_bytes()).unwrap();
    }
    let result = Execution::default().run();
    env::set_current_dir(original_working_directory).unwrap();
    assert!(result.is_ok());
    package.child(WIX_PATH.as_path()).assert(predicate::path::exists());
    package.child(MAIN_WXS_PATH.as_path()).assert(predicate::path::exists());
    package.child(LICENSE_RTF_PATH.as_path()).assert(predicate::path::exists());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:File[@Id='LicenseFile']/@Name"
    ), LICENSE_FILE_NAME.to_owned());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:File[@Id='LicenseFile']/@Source"
    ), LICENSE_RTF.to_owned());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:WixVariable[@Id='WixUILicenseRtf']/@Value"
    ), LICENSE_RTF.to_owned());
}

#[test]
fn apache2_license_id_works() {
    let original_working_directory = env::current_dir().unwrap();
    let package = common::create_test_package();
    env::set_current_dir(package.path()).unwrap();
    let package_manifest = package.child("Cargo.toml");
    let mut toml: Value = {
        let mut cargo_toml_handle = File::open(package_manifest.path()).unwrap();
        let mut cargo_toml_content = String::new();
        cargo_toml_handle.read_to_string(&mut cargo_toml_content).unwrap();
        toml::from_str(&cargo_toml_content).unwrap()
    };
    {
        toml.get_mut("package").and_then(|p| {
            match p {
                Value::Table(ref mut t) => t.insert(String::from("license"), Value::from("Apache-2.0")),
                _ => panic!("The 'package' section is not a table"),
            };
            Some(p)
        }).expect("A package section for the Cargo.toml");
        let toml_string = toml.to_string();
        let mut cargo_toml_handle = File::create(package_manifest.path()).unwrap();
        cargo_toml_handle.write_all(toml_string.as_bytes()).unwrap();
    }
    let result = Execution::default().run();
    env::set_current_dir(original_working_directory).unwrap();
    assert!(result.is_ok());
    package.child(WIX_PATH.as_path()).assert(predicate::path::exists());
    package.child(MAIN_WXS_PATH.as_path()).assert(predicate::path::exists());
    package.child(LICENSE_RTF_PATH.as_path()).assert(predicate::path::exists());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:File[@Id='LicenseFile']/@Name"
    ), LICENSE_FILE_NAME.to_owned());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:File[@Id='LicenseFile']/@Source"
    ), LICENSE_RTF.to_owned());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:WixVariable[@Id='WixUILicenseRtf']/@Value"
    ), LICENSE_RTF.to_owned());
}

#[test]
fn gpl3_license_id_works() {
    let original_working_directory = env::current_dir().unwrap();
    let package = common::create_test_package();
    env::set_current_dir(package.path()).unwrap();
    let package_manifest = package.child("Cargo.toml");
    let mut toml: Value = {
        let mut cargo_toml_handle = File::open(package_manifest.path()).unwrap();
        let mut cargo_toml_content = String::new();
        cargo_toml_handle.read_to_string(&mut cargo_toml_content).unwrap();
        toml::from_str(&cargo_toml_content).unwrap()
    };
    {
        toml.get_mut("package").and_then(|p| {
            match p {
                Value::Table(ref mut t) => t.insert(String::from("license"), Value::from("GPL-3.0")),
                _ => panic!("The 'package' section is not a table"),
            };
            Some(p)
        }).expect("A package section for the Cargo.toml");
        let toml_string = toml.to_string();
        let mut cargo_toml_handle = File::create(package_manifest.path()).unwrap();
        cargo_toml_handle.write_all(toml_string.as_bytes()).unwrap();
    }
    let result = Execution::default().run();
    env::set_current_dir(original_working_directory).unwrap();
    assert!(result.is_ok());
    package.child(WIX_PATH.as_path()).assert(predicate::path::exists());
    package.child(MAIN_WXS_PATH.as_path()).assert(predicate::path::exists());
    package.child(LICENSE_RTF_PATH.as_path()).assert(predicate::path::exists());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:File[@Id='LicenseFile']/@Name"
    ), LICENSE_FILE_NAME.to_owned());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:File[@Id='LicenseFile']/@Source"
    ), LICENSE_RTF.to_owned());
    assert_eq!(common::evaluate_xpath(
        package.child(MAIN_WXS_PATH.as_path()).path(),
        "//*/wix:WixVariable[@Id='WixUILicenseRtf']/@Value"
    ), LICENSE_RTF.to_owned());
}
