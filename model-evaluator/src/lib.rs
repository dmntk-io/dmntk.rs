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

#[macro_use]
extern crate dmntk_macros;

mod boxed_expressions;
mod business_knowledge_model;
mod decision;
mod decision_service;
mod decision_table;
mod errors;
mod input_data;
mod input_data_context;
mod item_definition;
mod item_definition_context;
mod item_definition_type;
mod model_builder;
mod model_definitions;
mod model_evaluator;
mod type_ref;
mod variable;

#[cfg(test)]
mod tests;

pub use decision_table::build_decision_table_evaluator;
pub use model_evaluator::ModelEvaluator;

#[cfg(test)]
mod utilities {
    use dmntk_common_1::{color_red, color_reset, ColorMode};
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use walkdir::WalkDir;

    /// This is a utility function for comparing the number of compatibility tests
    /// with the number of compatibility benchmarks, used to verify if all tests
    /// are also benchmarked.
    #[test]
    #[ignore]
    fn compare_the_number_of_tests_and_benchmarks() {
        let (tests, test_count) = count_number_of_artefacts("src/tests/compatibility", "#[test]");
        let (benches, bench_count) = count_number_of_artefacts("benches/compatibility", "#[bench]");
        println!("---------------------------------------------------------------");
        println!("      tests: {}", test_count);
        println!(" benchmarks: {}", bench_count);
        println!("---------------------------------------------------------------");
        println!(
            " {:29} {:^9} {:^6} {:^6}",
            "File name", "Tests", "Benchmarks", "Difference"
        );
        println!("---------------------------------------------------------------");
        let mut file_names = BTreeSet::new();
        file_names.append(
            &mut tests
                .keys()
                .map(|key| key.to_string())
                .collect::<BTreeSet<String>>(),
        );
        file_names.append(
            &mut benches
                .keys()
                .map(|key| key.to_string())
                .collect::<BTreeSet<String>>(),
        );
        for file_name in &file_names {
            let t_count = tests.get(file_name).unwrap_or(&0_usize).to_owned();
            let b_count = benches.get(file_name).unwrap_or(&0_usize).to_owned();
            println!(
                "{:30} {:>6} {:>12} {}",
                file_name,
                t_count,
                b_count,
                diff(t_count, b_count)
            );
        }
        println!("---------------------------------------------------------------");
        println!(
            "{:30} {:>6} {:>12} {}",
            "Total",
            test_count,
            bench_count,
            diff(test_count, bench_count)
        );
    }

    /// Counts the number of specific words in a file.
    fn count_number_of_artefacts(
        root_dir: &str,
        artefact: &str,
    ) -> (BTreeMap<String, usize>, usize) {
        let mut results = BTreeMap::new();
        let mut total = 0_usize;
        for entry_result in WalkDir::new(root_dir).into_iter() {
            match entry_result {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                        let content = fs::read_to_string(path).unwrap_or_else(|e| {
                            panic!(
                                "failed to load file: {} with reason: {}",
                                path.display(),
                                e.to_string()
                            )
                        });
                        let mut count = 0_usize;
                        for line in content.lines() {
                            if line.trim() == artefact {
                                count += 1;
                            }
                        }
                        if count > 1 {
                            results.insert(
                                path.strip_prefix(root_dir).unwrap().display().to_string(),
                                count,
                            );
                            total += count;
                        }
                    }
                }
                Err(reason) => eprintln!("{}", reason.to_string()),
            }
        }
        (results, total)
    }

    /// Prepares the difference in text format.
    fn diff(a: usize, b: usize) -> String {
        let color_red = color_red!(ColorMode::On);
        let color_reset = color_reset!(ColorMode::On);
        let diff = if a > b { a - b } else { b - a };
        if diff > 0 {
            format!("{1}{2:>9}{0}", color_reset, color_red, diff)
        } else {
            "".to_string()
        }
    }
}
