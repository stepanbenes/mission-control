// Stick
// Copyright © 2017-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your option (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).  This file may not be copied,
// modified, or distributed except according to those terms.

/// Window grab focus, re-enable events if they were disabled.
pub fn focus() {
    crate::stick::raw::GLOBAL.with(|g| g.enable());
}

/// Window ungrab focus, disable events.
pub fn unfocus() {
    crate::stick::raw::GLOBAL.with(|g| g.disable());
}
