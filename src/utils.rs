/*
 * Copyright (C) 2026 Ihit Rajesh Acharya
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY.
 */
pub mod utils {
    #[allow(non_snake_case, non_camel_case_types)]
    pub fn DBG_STR(inject: &str) -> String {
        format!(
            "****** DEBUG ******\nFile: {}\nLine: {}\nCol: {}\n{}*******************\n",
            file!(),
            line!(),
            column!(),
            inject
        )
    }
}
