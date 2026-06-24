//! Color system. Hue encodes node *type*; brightness encodes *echelon depth*.
//! See THEME.md for the contract.

use crate::anim;
use ratatui::style::Color;

/// Base RGB for each node type, grouped into hue families.
pub fn base_rgb(org_type: &str) -> (u8, u8, u8) {
    match org_type {
        // Apex / gold
        "department" => (255, 205, 70),
        "principal" => (255, 205, 70),
        // Secretariat / blue
        "osd" => (91, 138, 192),
        "assistant-secretary" => (100, 160, 220),
        "deputy-assistant-secretary" => (140, 185, 230),
        // Joint / violet
        "joint" => (155, 127, 224),
        // Service / green
        "mildep" => (80, 165, 105),
        "service" => (150, 205, 140),
        // Combatant commands — geo stays amber-orange; func goes crimson for contrast
        "cocom-geo" => (235, 148, 50),
        "cocom-func" => (215, 55, 55),
        // Force providers / teal — slightly brighter for readability
        "major-command" => (50, 190, 185),
        "command" => (85, 215, 215),
        // Defense enterprise / slate — lifted a touch so they don't grey-out
        "agency" => (145, 162, 185),
        "field-activity" => (115, 172, 182),
        "center" => (125, 185, 168),
        "lab" => (135, 195, 205),
        "directorate" => (155, 168, 188),
        _ => (120, 120, 120),
    }
}

/// Human-readable label for a node type.
pub fn type_label(org_type: &str) -> &'static str {
    match org_type {
        "department" => "Department (apex)",
        "principal" => "Secretary / Deputy",
        "osd" => "OSD staff",
        "assistant-secretary" => "Assistant Secretary",
        "deputy-assistant-secretary" => "Deputy Assistant Secretary",
        "joint" => "Joint (CJCS / Joint Staff)",
        "mildep" => "Military Department",
        "service" => "Armed Service",
        "cocom-geo" => "COCOM — geographic",
        "cocom-func" => "COCOM — functional",
        "major-command" => "Major Command",
        "command" => "Command",
        "agency" => "Defense Agency",
        "field-activity" => "Field Activity",
        "center" => "Center",
        "lab" => "Laboratory",
        "directorate" => "Directorate",
        _ => "Unit",
    }
}

/// Brightness multiplier by echelon — apex bright, depth dimmer.
pub fn shade_factor(echelon: u8) -> f32 {
    let e = (echelon as f32 / 5.0).min(1.0);
    anim::lerp(1.15, 0.70, e)
}

fn scale((r, g, b): (u8, u8, u8), f: f32) -> Color {
    let c = |x: u8| (x as f32 * f).round().clamp(0.0, 255.0) as u8;
    Color::Rgb(c(r), c(g), c(b))
}

/// Color for a node at rest.
pub fn node_color(org_type: &str, echelon: u8) -> Color {
    scale(base_rgb(org_type), shade_factor(echelon))
}

/// Color for a node with an extra brightness factor (boot fade-in, glow).
pub fn node_color_factor(org_type: &str, echelon: u8, extra: f32) -> Color {
    scale(base_rgb(org_type), shade_factor(echelon) * extra)
}

/// Base RGB for an operational edge, keyed by relation (see THEME.md).
pub fn relation_rgb(relation: &str) -> (u8, u8, u8) {
    match relation {
        "provides_forces_to" => (232, 140, 60),       // force flow = orange
        "service_component_of" => (90, 205, 210),     // component = cyan
        "combat_support_agency_for" => (120, 200, 140), // support = green
        "oversight_by" => (175, 150, 230),            // oversight = violet
        "reports_operationally_to" => (235, 205, 90), // reporting = gold
        _ => (170, 178, 190),
    }
}

/// Human-readable relation label.
pub fn relation_label(relation: &str) -> &'static str {
    match relation {
        "provides_forces_to" => "provides forces to",
        "service_component_of" => "service component of",
        "combat_support_agency_for" => "combat support agency for",
        "oversight_by" => "oversight by",
        "reports_operationally_to" => "reports operationally to",
        _ => "related to",
    }
}

/// Scale an RGB triple by a brightness factor into a Color.
pub fn rgb_scale((r, g, b): (u8, u8, u8), f: f32) -> Color {
    scale((r, g, b), f)
}

/// Banner color, driven by classification (never hard-coded to one level).
pub fn classification_color(classification: &str) -> Color {
    let c = classification.to_ascii_uppercase();
    if c.contains("TOP SECRET") {
        Color::Rgb(240, 120, 30)
    } else if c.contains("SECRET") {
        Color::Rgb(220, 60, 60)
    } else if c.contains("CONFIDENTIAL") {
        Color::Rgb(70, 130, 220)
    } else {
        Color::Rgb(0, 200, 0) // UNCLASSIFIED
    }
}
