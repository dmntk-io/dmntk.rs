/*
 * DMNTK - Decision Model and Notation Toolkit
 *
 * MIT license
 *
 * Copyright (c) 2015-2023 Dariusz Depta, Engos Software
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
 * Copyright (c) 2015-2023 Dariusz Depta, Engos Software
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

//! # Decision model validator
//!
//! Validations at the single decision model level:
//!
//! - Cycles in item definitions.
//!
//! TO-DO:
//!
//! - Go through the spec and add all required cycle checks on single model level.
//!

use crate::errors::err_item_definitions_cycle;
use crate::{Definitions, ItemDefinition, NamedElement};
use dmntk_common::Result;
use petgraph::algo::is_cyclic_directed;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Validates the decision model.
pub fn validate(definitions: Definitions) -> Result<Definitions> {
  let mut model_validator = ModelValidator::new();
  model_validator.validate(definitions)
}

/// Decision model validator.
pub struct ModelValidator {
  /// Directed graph for modelling item definition type references.
  item_definition_graph: DiGraph<String, &'static str>,
  item_definition_index: HashMap<String, NodeIndex>,
}

impl ModelValidator {
  /// Creates new model validator.
  fn new() -> Self {
    Self {
      item_definition_graph: DiGraph::new(),
      item_definition_index: HashMap::new(),
    }
  }

  /// Validated the decision model.
  fn validate(&mut self, definitions: Definitions) -> Result<Definitions> {
    self.check_recursive_item_definitions(&definitions)?;
    Ok(definitions)
  }

  /// Checks if there are no recursive item definitions.
  /// Recursive item definitions are not allowed in DMN 1.3 specification
  fn check_recursive_item_definitions(&mut self, definitions: &Definitions) -> Result<()> {
    for item_definition in &definitions.item_definitions {
      let name = item_definition.name().to_string();
      let node_index = self.item_definition_graph.add_node(name.clone());
      self.item_definition_index.insert(name.clone(), node_index);
      self.check_recursive_item_definition(name, node_index, item_definition);
    }
    if is_cyclic_directed(&self.item_definition_graph) {
      Err(err_item_definitions_cycle())
    } else {
      Ok(())
    }
  }

  /// Traverses item definition and registers dependencies.
  fn check_recursive_item_definition(&mut self, parent_name: String, parent_node_index: NodeIndex, item_definition: &ItemDefinition) {
    if let Some(type_ref) = &item_definition.type_ref {
      if let Some(ref_node_index) = self.item_definition_index.get(type_ref) {
        self.item_definition_graph.add_edge(parent_node_index, *ref_node_index, "type reference");
      } else {
        let ref_node_index = self.item_definition_graph.add_node(type_ref.to_string());
        self.item_definition_index.insert(type_ref.to_string(), ref_node_index);
        self.item_definition_graph.add_edge(parent_node_index, ref_node_index, "type reference");
      }
    }
    for component_item_definition in &item_definition.item_components {
      let component_name = format!("{}.{}", parent_name, component_item_definition.name());
      let component_node_index = self.item_definition_graph.add_node(component_name.clone());
      self.item_definition_graph.add_edge(parent_node_index, component_node_index, "component reference");
      self.check_recursive_item_definition(component_name, component_node_index, component_item_definition);
    }
  }
}
