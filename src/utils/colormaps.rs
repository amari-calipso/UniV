use raylib::color::Color;

use crate::rgb_color;

// https://matplotlib.org/stable/users/explain/colors/colormaps.html
pub const MAGMA: &[Color] = &[
    rgb_color!(43, 17, 94), 
    rgb_color!(45, 16, 96), 
    rgb_color!(47, 16, 98), 
    rgb_color!(48, 16, 101), 
    rgb_color!(48, 16, 101), 
    rgb_color!(50, 16, 103), 
    rgb_color!(52, 16, 104), 
    rgb_color!(53, 15, 106), 
    rgb_color!(55, 15, 108), 
    rgb_color!(55, 15, 108), 
    rgb_color!(57, 15, 110), 
    rgb_color!(59, 15, 111), 
    rgb_color!(60, 15, 113), 
    rgb_color!(62, 15, 114), 
    rgb_color!(64, 15, 115), 
    rgb_color!(64, 15, 115), 
    rgb_color!(66, 15, 116), 
    rgb_color!(67, 15, 117), 
    rgb_color!(69, 15, 118), 
    rgb_color!(71, 15, 119), 
    rgb_color!(71, 15, 119), 
    rgb_color!(72, 16, 120), 
    rgb_color!(74, 16, 121), 
    rgb_color!(75, 16, 121), 
    rgb_color!(77, 17, 122), 
    rgb_color!(79, 17, 123), 
    rgb_color!(79, 17, 123), 
    rgb_color!(80, 18, 123), 
    rgb_color!(82, 18, 124), 
    rgb_color!(83, 19, 124), 
    rgb_color!(85, 19, 125), 
    rgb_color!(85, 19, 125), 
    rgb_color!(87, 20, 125), 
    rgb_color!(88, 21, 126), 
    rgb_color!(90, 21, 126), 
    rgb_color!(91, 22, 126), 
    rgb_color!(91, 22, 126), 
    rgb_color!(93, 23, 126), 
    rgb_color!(94, 23, 127), 
    rgb_color!(96, 24, 127), 
    rgb_color!(97, 24, 127), 
    rgb_color!(99, 25, 127), 
    rgb_color!(99, 25, 127), 
    rgb_color!(101, 26, 128), 
    rgb_color!(102, 26, 128), 
    rgb_color!(104, 27, 128), 
    rgb_color!(105, 28, 128), 
    rgb_color!(105, 28, 128), 
    rgb_color!(107, 28, 128), 
    rgb_color!(108, 29, 128), 
    rgb_color!(110, 30, 129), 
    rgb_color!(111, 30, 129), 
    rgb_color!(111, 30, 129), 
    rgb_color!(113, 31, 129), 
    rgb_color!(115, 31, 129), 
    rgb_color!(116, 32, 129), 
    rgb_color!(118, 33, 129), 
    rgb_color!(119, 33, 129), 
    rgb_color!(119, 33, 129), 
    rgb_color!(121, 34, 129), 
    rgb_color!(122, 34, 129), 
    rgb_color!(124, 35, 129), 
    rgb_color!(126, 36, 129), 
    rgb_color!(126, 36, 129), 
    rgb_color!(127, 36, 129), 
    rgb_color!(129, 37, 129), 
    rgb_color!(130, 37, 129), 
    rgb_color!(132, 38, 129), 
    rgb_color!(133, 38, 129), 
    rgb_color!(133, 38, 129), 
    rgb_color!(135, 39, 129), 
    rgb_color!(137, 40, 129), 
    rgb_color!(138, 40, 129), 
    rgb_color!(140, 41, 128), 
    rgb_color!(140, 41, 128), 
    rgb_color!(141, 41, 128), 
    rgb_color!(143, 42, 128), 
    rgb_color!(145, 42, 128), 
    rgb_color!(146, 43, 128), 
    rgb_color!(146, 43, 128), 
    rgb_color!(148, 43, 128), 
    rgb_color!(149, 44, 128), 
    rgb_color!(151, 44, 127), 
    rgb_color!(153, 45, 127), 
    rgb_color!(154, 45, 127), 
    rgb_color!(154, 45, 127), 
    rgb_color!(156, 46, 127), 
    rgb_color!(158, 46, 126), 
    rgb_color!(159, 47, 126), 
    rgb_color!(161, 47, 126), 
    rgb_color!(161, 47, 126), 
    rgb_color!(163, 48, 126), 
    rgb_color!(164, 48, 125), 
    rgb_color!(166, 49, 125), 
    rgb_color!(167, 49, 125), 
    rgb_color!(167, 49, 125), 
    rgb_color!(169, 50, 124), 
    rgb_color!(171, 51, 124), 
    rgb_color!(172, 51, 123), 
    rgb_color!(174, 52, 123), 
    rgb_color!(176, 52, 123), 
    rgb_color!(176, 52, 123), 
    rgb_color!(177, 53, 122), 
    rgb_color!(179, 53, 122), 
    rgb_color!(181, 54, 121), 
    rgb_color!(182, 54, 121), 
    rgb_color!(182, 54, 121), 
    rgb_color!(184, 55, 120), 
    rgb_color!(185, 55, 120), 
    rgb_color!(187, 56, 119), 
    rgb_color!(189, 57, 119), 
    rgb_color!(190, 57, 118), 
    rgb_color!(190, 57, 118), 
    rgb_color!(192, 58, 117), 
    rgb_color!(194, 58, 117), 
    rgb_color!(195, 59, 116), 
    rgb_color!(197, 60, 116), 
    rgb_color!(197, 60, 116), 
    rgb_color!(198, 60, 115), 
    rgb_color!(200, 61, 114), 
    rgb_color!(202, 62, 114), 
    rgb_color!(203, 62, 113), 
    rgb_color!(203, 62, 113), 
    rgb_color!(205, 63, 112), 
    rgb_color!(206, 64, 112), 
    rgb_color!(208, 65, 111), 
    rgb_color!(209, 66, 110), 
    rgb_color!(211, 66, 109), 
    rgb_color!(211, 66, 109), 
    rgb_color!(212, 67, 109), 
    rgb_color!(214, 68, 108), 
    rgb_color!(215, 69, 107), 
    rgb_color!(217, 70, 106), 
    rgb_color!(217, 70, 106), 
    rgb_color!(218, 71, 105), 
    rgb_color!(220, 72, 105), 
    rgb_color!(221, 73, 104), 
    rgb_color!(222, 74, 103), 
    rgb_color!(222, 74, 103), 
    rgb_color!(224, 75, 102), 
    rgb_color!(225, 76, 102), 
    rgb_color!(226, 77, 101), 
    rgb_color!(228, 78, 100), 
    rgb_color!(229, 80, 99), 
    rgb_color!(229, 80, 99), 
    rgb_color!(230, 81, 98), 
    rgb_color!(231, 82, 98), 
    rgb_color!(232, 84, 97), 
    rgb_color!(234, 85, 96), 
    rgb_color!(234, 85, 96), 
    rgb_color!(235, 86, 96), 
    rgb_color!(236, 88, 95), 
    rgb_color!(237, 89, 95), 
    rgb_color!(238, 91, 94), 
    rgb_color!(238, 93, 93), 
    rgb_color!(238, 93, 93), 
    rgb_color!(239, 94, 93), 
    rgb_color!(240, 96, 93), 
    rgb_color!(241, 97, 92), 
    rgb_color!(242, 99, 92), 
    rgb_color!(242, 99, 92), 
    rgb_color!(243, 101, 92), 
    rgb_color!(243, 103, 91), 
    rgb_color!(244, 104, 91), 
    rgb_color!(245, 106, 91), 
    rgb_color!(245, 106, 91), 
    rgb_color!(245, 108, 91), 
    rgb_color!(246, 110, 91), 
    rgb_color!(246, 112, 91), 
    rgb_color!(247, 113, 91), 
    rgb_color!(247, 115, 92), 
    rgb_color!(247, 115, 92), 
    rgb_color!(248, 117, 92), 
    rgb_color!(248, 119, 92), 
    rgb_color!(249, 121, 92), 
    rgb_color!(249, 123, 93), 
    rgb_color!(249, 123, 93), 
    rgb_color!(249, 125, 93), 
    rgb_color!(250, 127, 94), 
    rgb_color!(250, 128, 94), 
    rgb_color!(250, 130, 95), 
    rgb_color!(250, 130, 95), 
    rgb_color!(251, 132, 96), 
    rgb_color!(251, 134, 96), 
    rgb_color!(251, 136, 97), 
    rgb_color!(251, 138, 98), 
    rgb_color!(252, 140, 99), 
    rgb_color!(252, 140, 99), 
    rgb_color!(252, 142, 99), 
    rgb_color!(252, 144, 100), 
    rgb_color!(252, 146, 101), 
    rgb_color!(252, 147, 102), 
    rgb_color!(252, 147, 102), 
    rgb_color!(253, 149, 103), 
    rgb_color!(253, 151, 104), 
    rgb_color!(253, 153, 105), 
    rgb_color!(253, 155, 106), 
    rgb_color!(253, 157, 107), 
    rgb_color!(253, 157, 107), 
    rgb_color!(253, 159, 108), 
    rgb_color!(253, 161, 110), 
    rgb_color!(253, 162, 111), 
    rgb_color!(253, 164, 112), 
    rgb_color!(253, 164, 112), 
    rgb_color!(254, 166, 113), 
    rgb_color!(254, 168, 115), 
    rgb_color!(254, 170, 116), 
    rgb_color!(254, 172, 117), 
    rgb_color!(254, 172, 117), 
    rgb_color!(254, 174, 118), 
    rgb_color!(254, 175, 120), 
    rgb_color!(254, 177, 121), 
    rgb_color!(254, 179, 123), 
    rgb_color!(254, 181, 124), 
    rgb_color!(254, 181, 124), 
    rgb_color!(254, 183, 125), 
    rgb_color!(254, 185, 127), 
    rgb_color!(254, 187, 128), 
    rgb_color!(254, 188, 130), 
    rgb_color!(254, 188, 130), 
    rgb_color!(254, 190, 131), 
    rgb_color!(254, 192, 133), 
    rgb_color!(254, 194, 134), 
    rgb_color!(254, 196, 136), 
    rgb_color!(254, 196, 136), 
    rgb_color!(254, 198, 137), 
    rgb_color!(254, 199, 139), 
    rgb_color!(254, 201, 141), 
    rgb_color!(254, 203, 142), 
    rgb_color!(253, 205, 144), 
    rgb_color!(253, 205, 144), 
    rgb_color!(253, 207, 146), 
    rgb_color!(253, 209, 147), 
    rgb_color!(253, 210, 149), 
    rgb_color!(253, 212, 151), 
    rgb_color!(253, 212, 151), 
    rgb_color!(253, 214, 152), 
    rgb_color!(253, 216, 154), 
    rgb_color!(253, 218, 156), 
    rgb_color!(253, 220, 157), 
    rgb_color!(253, 221, 159), 
    rgb_color!(253, 221, 159), 
    rgb_color!(253, 223, 161), 
    rgb_color!(253, 225, 163), 
    rgb_color!(252, 227, 165), 
    rgb_color!(252, 229, 166), 
    rgb_color!(252, 229, 166), 
    rgb_color!(252, 230, 168), 
    rgb_color!(252, 232, 170), 
    rgb_color!(252, 234, 172), 
    rgb_color!(252, 234, 172)
];