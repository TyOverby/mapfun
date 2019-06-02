use crate::svg_exporter::*;
use crate::*;

pub fn puke_theme(svg: &mut Svg<Layer>) {
    svg.set_background_color("#1f2345");

    svg.set_style(
        Layer::Road,
        "road",
        "fill:none; stroke:#8b8ca9; stroke-width:0.07%; stroke-linecap:round",
    );

    svg.set_style(
        Layer::Building,
        "building",
        "fill:#dc9433; stroke:#000; stroke-width:0.01px",
    );

    svg.set_style(
        Layer::ParkBuilding,
        "park-building",
        "fill:#ff0000; stroke:#f44336; stroke-width:0.1px",
    );

    svg.set_style(
        Layer::ParkPath,
        "park-path",
        "fill:none; stroke:#e841f4; stroke-width:0.01px",
    );

    svg.set_style(
        Layer::Coastline,
        "coastline",
        "fill:#eee; stroke:white; stroke-width:1px",
    );

    svg.set_style(
        Layer::Subway,
        "road",
        "fill:none; stroke:#ff0000; stroke-width:0.07%; stroke-linecap:round",
    );

    svg.set_style(Layer::Park, "park", "fill:#42f442; stroke:none;");
}

pub fn gray_theme(svg: &mut Svg<Layer>) {
    svg.set_background_color("#fff");

    svg.set_style(
        Layer::Road,
        "road",
        "fill:none; stroke:#bbb; stroke-width:0.07%; stroke-linecap:round",
    );

    svg.set_style(Layer::Building, "building", "fill:#fff; stroke:none;");

    svg.set_style(
        Layer::ParkBuilding,
        "park-building",
        "fill:#777; stroke:none;",
    );

    svg.set_style(
        Layer::ParkPath,
        "park-path",
        "fill:none; stroke:#777; stroke-width:0.01px",
    );

    svg.set_style(
        Layer::Subway,
        "subway",
        "fill:none; stroke:#ff0000; stroke-width:0.3%; stroke-linecap:round",
    );

    svg.set_style(Layer::Coastline, "coastline", "fill:#777; stroke:none;");
    svg.set_style(Layer::Park, "park", "fill:#777; stroke:none;");
}
