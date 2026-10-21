//! Liquid bulk cargo: crude oil on tank barges and small coastal tankers (spec 0006, ADR 0016).
//!
//! The module grows ticket by ticket. Today it holds the Crude Grade data and the ADN Tank Vessel
//! Type derived from it. The layout, Spaces and free surface come with the next ticket.

pub mod adn_vessel_type;
pub mod crude_grade;
