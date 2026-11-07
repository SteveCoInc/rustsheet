#![allow(clippy::unwrap_used)]

use crate::{expressions::types::Area, types::Border, BorderArea, UserModel};

/// An empty en-US user model in UTC on the fixed test clock.
pub fn new_empty_user_model() -> UserModel {
    UserModel::from_model(crate::test::util::new_empty_model())
}

impl UserModel {
    pub fn set_cell_border_a1(&mut self, cell: &str, color: &str) {
        let cell_reference = self.model.parse_a1(cell);
        let column = cell_reference.column;
        let row = cell_reference.row;
        let border_area: BorderArea = serde_json::from_str(&format!(
            r#"{{
                "item": {{
                    "style": "thin",
                    "color": "{color}"
                }},
                "type": "All"
            }}"#
        ))
        .unwrap();
        let range = &Area {
            sheet: 0,
            row,
            column,
            width: 1,
            height: 1,
        };
        self.set_area_with_border(range, &border_area).unwrap();
    }

    pub fn set_area_border_a1(&mut self, range: &str, color: &str, kind: &str) {
        let s: Vec<&str> = range.split(':').collect();
        let left = self.model.parse_a1(s[0]);
        let right = self.model.parse_a1(s[1]);
        let column = left.column;
        let row = left.row;
        let width = right.column - column + 1;
        let height = right.row - row + 1;
        let border_area: BorderArea = serde_json::from_str(&format!(
            r#"{{
                "item": {{
                    "style": "thin",
                    "color": "{color}"
                }},
                "type": "{kind}"
            }}"#
        ))
        .unwrap();
        let range = &Area {
            sheet: 0,
            row,
            column,
            width,
            height,
        };
        self.set_area_with_border(range, &border_area).unwrap();
    }

    pub fn get_cell_border_a1(&self, cell: &str) -> Border {
        let cell_reference = self.model.parse_a1(cell);
        let column = cell_reference.column;
        let row = cell_reference.row;
        let style = self.get_cell_style(0, row, column).unwrap();
        style.border
    }

    pub fn get_cell_actual_border_a1(&self, cell: &str) -> Border {
        let cell_reference = self.model.parse_a1(cell);
        let column = cell_reference.column;
        let row = cell_reference.row;
        let style = self.model.get_style_for_cell(0, row, column).unwrap();
        style.border
    }
}
