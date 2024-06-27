/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

pub mod badge;
pub mod card;
pub mod form;
pub mod icon;
pub mod layout;
pub mod list;
pub mod messages;
pub mod report;
pub mod skeleton;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Blue,
    Gray,
    Red,
    Yellow,
    Green,
}
