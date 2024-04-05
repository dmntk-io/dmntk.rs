/*
 * DMNTK - Decision Model and Notation Toolkit
 *
 * MIT license
 *
 * Copyright (c) 2015-2023 Dariusz Depta
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 * Apache license, Version 2.0
 *
 * Copyright (c) 2015-2023 Dariusz Depta
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Parsing tables generator for `LALR` parser written in Rust.

use std::fs::{self, File};
use std::io::Write;

/// Holds the content of `FEEL` grammar in Bison compatible format.
const FEEL_GRAMMAR: &str = include_str!("feel.y");

/// Holds the content of grammar generation script.
const GEN_SCRIPT: &str = include_str!("gen.sh");

/// Name of the grammar file.
const GRAMMAR_FILE_NAME: &str = "feel.y";

/// Name of the generation script file.
const GEN_SCRIPT_FILE_NAME: &str = "gen.sh";

/// Name of the parser tables file.
const TABLES_FILE_NAME: &str = "feel.tab.c";

/// Name of the target directory.
const TARGET_DIR: &str = "../target/feel-grammar";

/// Sets file permissions for Linux system.
#[cfg(target_os = "linux")]
fn set_file_permissions(file_name: &str) {
  use std::fs::Permissions;
  use std::os::unix::fs::PermissionsExt;
  fs::set_permissions(file_name, Permissions::from_mode(0o755)).expect("setting script permissions failed");
}

/// Sets file permissions for non-Linux systems.
#[cfg(not(target_os = "linux"))]
fn set_file_permissions(_file_name: &str) {
  // do nothing
}

/// Returns the source code of the parsing tables for `C` language generated by `Bison` parser generator.
fn lalr_c_tables() -> String {
  {
    // create required directory structure in `target`
    fs::create_dir_all(TARGET_DIR).expect("creating target directories failed");
    // create the grammar file
    let grammar_file_name = format!("{TARGET_DIR}/{GRAMMAR_FILE_NAME}");
    let mut grammar_file = File::create(grammar_file_name).expect("creating grammar file failed");
    grammar_file.write_all(FEEL_GRAMMAR.as_bytes()).expect("writing grammar file failed");
    // create the 'C' grammar generation script
    let script_file_name = format!("{TARGET_DIR}/{GEN_SCRIPT_FILE_NAME}");
    let mut script_file = File::create(&script_file_name).expect("creating script file failed");
    script_file.write_all(GEN_SCRIPT.as_bytes()).expect("writing script file failed");
    set_file_permissions(&script_file_name);
  }
  {
    let mut command_process = std::process::Command::new(format!("./{GEN_SCRIPT_FILE_NAME}"))
      .current_dir(TARGET_DIR)
      .spawn()
      .expect("executing script failed");
    command_process.wait().expect("waiting for command process failed");
  }
  fs::read_to_string(format!("{TARGET_DIR}/{TABLES_FILE_NAME}")).expect("generating parsing tables failed")
}

/// Writes to file the source code of parsing tables for `Rust` language,
/// extracted from parsing tables generated by `Bison` parser generator
/// for `C` language.
///
/// # Examples
///
/// ```no_run
/// use dmntk_feel_grammar::lalr_rust_tables;
///
/// // Parsing tables will be written to file named `lalr.rs` in `./src` directory.
/// lalr_rust_tables("./src/lalr.rs");
/// ```
pub fn lalr_rust_tables(output_file: &str) {
  let lalr_rust_tables = crate::extractor::extract(&lalr_c_tables());
  fs::write(output_file, lalr_rust_tables).expect("writing output file failed");
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Executes all tests sequentially, because these tests access the same files.
  #[test]
  fn test_all_sequentially() {
    if std::env::var("CI").is_err() {
      // run these tests only outside CI (GitHub Actions)
      test_feel_grammar();
      test_lalr_c_tables();
      test_lalr_rust_tables();
    }
  }

  fn test_feel_grammar() {
    let len = FEEL_GRAMMAR.len();
    assert!(len > 1000);
    assert_eq!("%start feel", &FEEL_GRAMMAR[1309..1320]);
    assert_eq!("%%\n", &FEEL_GRAMMAR[len - 3..]);
  }

  fn test_lalr_c_tables() {
    let lalr_c_tables = lalr_c_tables();
    assert!(lalr_c_tables.len() > 1000);
    assert_eq!("Bison", &lalr_c_tables[5..10]);
  }

  fn test_lalr_rust_tables() {
    let output_file = format!("{TARGET_DIR}/lalr.rs");
    lalr_rust_tables(&output_file);
    let lalr = fs::read_to_string(output_file).expect("reading Rust LALR tables failed");
    assert!(lalr.len() > 1000);
    assert_eq!("DMNTK", &lalr[6..11]);
  }
}
