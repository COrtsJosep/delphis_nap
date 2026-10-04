use plotters::style::RGBAColor;

pub fn fetch_palette(num_required_colours: usize) -> Vec<RGBAColor> {
    if num_required_colours <= 8 {
        vec![
            // QPBI Palette
            // Source: The R package: {Redmonder}
            RGBAColor(0, 184, 170, 1.0),
            RGBAColor(55, 70, 73, 1.0),
            RGBAColor(253, 98, 94, 1.0),
            RGBAColor(242, 200, 17, 1.0),
            RGBAColor(95, 107, 109, 1.0),
            RGBAColor(138, 212, 235, 1.0),
            RGBAColor(254, 150, 102, 1.0),
            RGBAColor(166, 105, 153, 1.0),
        ]
    } else if num_required_colours <= 12 {
        vec![
            // Vivid Palette
            // Source: The R package: {rcartocolor}
            RGBAColor(229, 134, 6, 1.0),
            RGBAColor(93, 105, 177, 1.0),
            RGBAColor(82, 188, 163, 1.0),
            RGBAColor(153, 201, 69, 1.0),
            RGBAColor(204, 97, 176, 1.0),
            RGBAColor(36, 121, 108, 1.0),
            RGBAColor(218, 165, 27, 1.0),
            RGBAColor(47, 138, 196, 1.0),
            RGBAColor(118, 78, 159, 1.0),
            RGBAColor(237, 100, 90, 1.0),
            RGBAColor(204, 58, 142, 1.0),
            RGBAColor(165, 170, 153, 1.0),
        ]
    } else {
        vec![
            // Gravity Falls Palette
            // Source: The R package: {tvthemes}
            RGBAColor(65, 123, 161, 1.0),
            RGBAColor(255, 20, 147, 1.0),
            RGBAColor(255, 255, 46, 1.0),
            RGBAColor(52, 86, 52, 1.0),
            RGBAColor(139, 0, 0, 1.0),
            RGBAColor(255, 103, 0, 1.0),
            RGBAColor(147, 192, 213, 1.0),
            RGBAColor(139, 69, 19, 1.0),
            RGBAColor(146, 72, 167, 1.0),
            RGBAColor(28, 136, 89, 1.0),
            RGBAColor(71, 71, 71, 1.0),
            RGBAColor(143, 188, 143, 1.0),
            RGBAColor(210, 180, 140, 1.0),
            RGBAColor(0, 0, 0, 1.0),
        ]
    }
}
