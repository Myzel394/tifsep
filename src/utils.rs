pub mod utils {
    use std::str::Utf8Error;

    use lazy_regex::regex_replace_all;
    use lazy_static::lazy_static;
    use regex::Regex;
    use urlencoding::decode;

    lazy_static! {
        static ref HTML_UNICODES: Regex = Regex::new(
            r#"&(?:(?<name>[a-zA-Z]+)|(?<code>#(?:(?<code_num>\d+)|(?<code_hex>x[0-9a-fA-F]+))));"#
        )
        .unwrap();
    }

    pub fn find_next_sequence(text: &str, sequence: &str, index_start: &usize) -> Option<usize> {
        for index in *index_start..text.len() {
            let new = &text[index..index + sequence.len()];

            if new == sequence {
                return Some(index);
            }
        }

        None
    }

    pub fn decode_html_text(str: &str) -> Result<String, Utf8Error> {
        let mut decoded = decode(str).unwrap().into_owned();
        Ok(replace_html_unicode(&mut decoded))
    }

    fn lookup_html_unicode_num(num: u32) -> Result<String, String> {
        match char::from_u32(num) {
            Some(value) => Ok(value.to_string()),
            None => Err("Symbol not found".to_string()),
        }
    }

    fn lookup_html_unicode_name(name: &str) -> Result<String, String> {
        // Entities are from https://www.w3.org/TR/html4/sgml/entities.html
        match name {
            "nbsp" => lookup_html_unicode_num(160), // no-break space = non-breaking space, U+00A0 ISOnum -->
            "iexcl" => lookup_html_unicode_num(161), // inverted exclamation mark, U+00A1 ISOnum -->
            "cent" => lookup_html_unicode_num(162), // cent sign, U+00A2 ISOnum -->
            "pound" => lookup_html_unicode_num(163), // pound sign, U+00A3 ISOnum -->
            "curren" => lookup_html_unicode_num(164), // currency sign, U+00A4 ISOnum -->
            "yen" => lookup_html_unicode_num(165),  // yen sign = yuan sign, U+00A5 ISOnum -->
            "brvbar" => lookup_html_unicode_num(166), // broken bar = broken vertical bar, U+00A6 ISOnum -->
            "sect" => lookup_html_unicode_num(167),   // section sign, U+00A7 ISOnum -->
            "uml" => lookup_html_unicode_num(168), // diaeresis = spacing diaeresis, U+00A8 ISOdia -->
            "copy" => lookup_html_unicode_num(169), // copyright sign, U+00A9 ISOnum -->
            "ordf" => lookup_html_unicode_num(170), // feminine ordinal indicator, U+00AA ISOnum -->
            "laquo" => lookup_html_unicode_num(171), // left-pointing double angle quotation mark = left pointing guillemet, U+00AB ISOnum -->
            "not" => lookup_html_unicode_num(172),   // not sign, U+00AC ISOnum -->
            "shy" => lookup_html_unicode_num(173), // soft hyphen = discretionary hyphen, U+00AD ISOnum -->
            "reg" => lookup_html_unicode_num(174), // registered sign = registered trade mark sign, U+00AE ISOnum -->
            "macr" => lookup_html_unicode_num(175), // macron = spacing macron = overline = APL overbar, U+00AF ISOdia -->
            "deg" => lookup_html_unicode_num(176),  // degree sign, U+00B0 ISOnum -->
            "plusmn" => lookup_html_unicode_num(177), // plus-minus sign = plus-or-minus sign, U+00B1 ISOnum -->
            "sup2" => lookup_html_unicode_num(178), // superscript two = superscript digit two = squared, U+00B2 ISOnum -->
            "sup3" => lookup_html_unicode_num(179), // superscript three = superscript digit three = cubed, U+00B3 ISOnum -->
            "acute" => lookup_html_unicode_num(180), // acute accent = spacing acute, U+00B4 ISOdia -->
            "micro" => lookup_html_unicode_num(181), // micro sign, U+00B5 ISOnum -->
            "para" => lookup_html_unicode_num(182), // pilcrow sign = paragraph sign, U+00B6 ISOnum -->
            "middot" => lookup_html_unicode_num(183), // middle dot = Georgian comma = Greek middle dot, U+00B7 ISOnum -->
            "cedil" => lookup_html_unicode_num(184), // cedilla = spacing cedilla, U+00B8 ISOdia -->
            "sup1" => lookup_html_unicode_num(185), // superscript one = superscript digit one, U+00B9 ISOnum -->
            "ordm" => lookup_html_unicode_num(186), // masculine ordinal indicator, U+00BA ISOnum -->
            "raquo" => lookup_html_unicode_num(187), // right-pointing double angle quotation mark = right pointing guillemet, U+00BB ISOnum -->
            "frac14" => lookup_html_unicode_num(188), // vulgar fraction one quarter = fraction one quarter, U+00BC ISOnum -->
            "frac12" => lookup_html_unicode_num(189), // vulgar fraction one half = fraction one half, U+00BD ISOnum -->
            "frac34" => lookup_html_unicode_num(190), // vulgar fraction three quarters = fraction three quarters, U+00BE ISOnum -->
            "iquest" => lookup_html_unicode_num(191), // inverted question mark = turned question mark, U+00BF ISOnum -->
            "Agrave" => lookup_html_unicode_num(192), // latin capital letter A with grave = latin capital letter A grave, U+00C0 ISOlat1 -->
            "Aacute" => lookup_html_unicode_num(193), // latin capital letter A with acute, U+00C1 ISOlat1 -->
            "Acirc" => lookup_html_unicode_num(194), // latin capital letter A with circumflex, U+00C2 ISOlat1 -->
            "Atilde" => lookup_html_unicode_num(195), // latin capital letter A with tilde, U+00C3 ISOlat1 -->
            "Auml" => lookup_html_unicode_num(196), // latin capital letter A with diaeresis, U+00C4 ISOlat1 -->
            "Aring" => lookup_html_unicode_num(197), // latin capital letter A with ring above = latin capital letter A ring, U+00C5 ISOlat1 -->
            "AElig" => lookup_html_unicode_num(198), // latin capital letter AE = latin capital ligature AE, U+00C6 ISOlat1 -->
            "Ccedil" => lookup_html_unicode_num(199), // latin capital letter C with cedilla, U+00C7 ISOlat1 -->
            "Egrave" => lookup_html_unicode_num(200), // latin capital letter E with grave, U+00C8 ISOlat1 -->
            "Eacute" => lookup_html_unicode_num(201), // latin capital letter E with acute, U+00C9 ISOlat1 -->
            "Ecirc" => lookup_html_unicode_num(202), // latin capital letter E with circumflex, U+00CA ISOlat1 -->
            "Euml" => lookup_html_unicode_num(203), // latin capital letter E with diaeresis, U+00CB ISOlat1 -->
            "Igrave" => lookup_html_unicode_num(204), // latin capital letter I with grave, U+00CC ISOlat1 -->
            "Iacute" => lookup_html_unicode_num(205), // latin capital letter I with acute, U+00CD ISOlat1 -->
            "Icirc" => lookup_html_unicode_num(206), // latin capital letter I with circumflex, U+00CE ISOlat1 -->
            "Iuml" => lookup_html_unicode_num(207), // latin capital letter I with diaeresis, U+00CF ISOlat1 -->
            "ETH" => lookup_html_unicode_num(208),  // latin capital letter ETH, U+00D0 ISOlat1 -->
            "Ntilde" => lookup_html_unicode_num(209), // latin capital letter N with tilde, U+00D1 ISOlat1 -->
            "Ograve" => lookup_html_unicode_num(210), // latin capital letter O with grave, U+00D2 ISOlat1 -->
            "Oacute" => lookup_html_unicode_num(211), // latin capital letter O with acute, U+00D3 ISOlat1 -->
            "Ocirc" => lookup_html_unicode_num(212), // latin capital letter O with circumflex, U+00D4 ISOlat1 -->
            "Otilde" => lookup_html_unicode_num(213), // latin capital letter O with tilde, U+00D5 ISOlat1 -->
            "Ouml" => lookup_html_unicode_num(214), // latin capital letter O with diaeresis, U+00D6 ISOlat1 -->
            "times" => lookup_html_unicode_num(215), // multiplication sign, U+00D7 ISOnum -->
            "Oslash" => lookup_html_unicode_num(216), // latin capital letter O with stroke = latin capital letter O slash, U+00D8 ISOlat1 -->
            "Ugrave" => lookup_html_unicode_num(217), // latin capital letter U with grave, U+00D9 ISOlat1 -->
            "Uacute" => lookup_html_unicode_num(218), // latin capital letter U with acute, U+00DA ISOlat1 -->
            "Ucirc" => lookup_html_unicode_num(219), // latin capital letter U with circumflex, U+00DB ISOlat1 -->
            "Uuml" => lookup_html_unicode_num(220), // latin capital letter U with diaeresis, U+00DC ISOlat1 -->
            "Yacute" => lookup_html_unicode_num(221), // latin capital letter Y with acute, U+00DD ISOlat1 -->
            "THORN" => lookup_html_unicode_num(222), // latin capital letter THORN, U+00DE ISOlat1 -->
            "szlig" => lookup_html_unicode_num(223), // latin small letter sharp s = ess-zed, U+00DF ISOlat1 -->
            "agrave" => lookup_html_unicode_num(224), // latin small letter a with grave = latin small letter a grave, U+00E0 ISOlat1 -->
            "aacute" => lookup_html_unicode_num(225), // latin small letter a with acute, U+00E1 ISOlat1 -->
            "acirc" => lookup_html_unicode_num(226), // latin small letter a with circumflex, U+00E2 ISOlat1 -->
            "atilde" => lookup_html_unicode_num(227), // latin small letter a with tilde, U+00E3 ISOlat1 -->
            "auml" => lookup_html_unicode_num(228), // latin small letter a with diaeresis, U+00E4 ISOlat1 -->
            "aring" => lookup_html_unicode_num(229), // latin small letter a with ring above = latin small letter a ring, U+00E5 ISOlat1 -->
            "aelig" => lookup_html_unicode_num(230), // latin small letter ae = latin small ligature ae, U+00E6 ISOlat1 -->
            "ccedil" => lookup_html_unicode_num(231), // latin small letter c with cedilla, U+00E7 ISOlat1 -->
            "egrave" => lookup_html_unicode_num(232), // latin small letter e with grave, U+00E8 ISOlat1 -->
            "eacute" => lookup_html_unicode_num(233), // latin small letter e with acute, U+00E9 ISOlat1 -->
            "ecirc" => lookup_html_unicode_num(234), // latin small letter e with circumflex, U+00EA ISOlat1 -->
            "euml" => lookup_html_unicode_num(235), // latin small letter e with diaeresis, U+00EB ISOlat1 -->
            "igrave" => lookup_html_unicode_num(236), // latin small letter i with grave, U+00EC ISOlat1 -->
            "iacute" => lookup_html_unicode_num(237), // latin small letter i with acute, U+00ED ISOlat1 -->
            "icirc" => lookup_html_unicode_num(238), // latin small letter i with circumflex, U+00EE ISOlat1 -->
            "iuml" => lookup_html_unicode_num(239), // latin small letter i with diaeresis, U+00EF ISOlat1 -->
            "eth" => lookup_html_unicode_num(240),  // latin small letter eth, U+00F0 ISOlat1 -->
            "ntilde" => lookup_html_unicode_num(241), // latin small letter n with tilde, U+00F1 ISOlat1 -->
            "ograve" => lookup_html_unicode_num(242), // latin small letter o with grave, U+00F2 ISOlat1 -->
            "oacute" => lookup_html_unicode_num(243), // latin small letter o with acute, U+00F3 ISOlat1 -->
            "ocirc" => lookup_html_unicode_num(244), // latin small letter o with circumflex, U+00F4 ISOlat1 -->
            "otilde" => lookup_html_unicode_num(245), // latin small letter o with tilde, U+00F5 ISOlat1 -->
            "ouml" => lookup_html_unicode_num(246), // latin small letter o with diaeresis, U+00F6 ISOlat1 -->
            "divide" => lookup_html_unicode_num(247), // division sign, U+00F7 ISOnum -->
            "oslash" => lookup_html_unicode_num(248), // latin small letter o with stroke, = latin small letter o slash, U+00F8 ISOlat1 -->
            "ugrave" => lookup_html_unicode_num(249), // latin small letter u with grave, U+00F9 ISOlat1 -->
            "uacute" => lookup_html_unicode_num(250), // latin small letter u with acute, U+00FA ISOlat1 -->
            "ucirc" => lookup_html_unicode_num(251), // latin small letter u with circumflex, U+00FB ISOlat1 -->
            "uuml" => lookup_html_unicode_num(252), // latin small letter u with diaeresis, U+00FC ISOlat1 -->
            "yacute" => lookup_html_unicode_num(253), // latin small letter y with acute, U+00FD ISOlat1 -->
            "thorn" => lookup_html_unicode_num(254), // latin small letter thorn, U+00FE ISOlat1 -->
            "yuml" => lookup_html_unicode_num(255), // latin small letter y with diaeresis, U+00FF ISOlat1 -->
            "fnof" => lookup_html_unicode_num(402), // latin small f with hook = function = florin, U+0192 ISOtech -->
            "Alpha" => lookup_html_unicode_num(913), // greek capital letter alpha, U+0391 -->
            "Beta" => lookup_html_unicode_num(914), // greek capital letter beta, U+0392 -->
            "Gamma" => lookup_html_unicode_num(915), // greek capital letter gamma, U+0393 ISOgrk3 -->
            "Delta" => lookup_html_unicode_num(916), // greek capital letter delta, U+0394 ISOgrk3 -->
            "Epsilon" => lookup_html_unicode_num(917), // greek capital letter epsilon, U+0395 -->
            "Zeta" => lookup_html_unicode_num(918),  // greek capital letter zeta, U+0396 -->
            "Eta" => lookup_html_unicode_num(919),   // greek capital letter eta, U+0397 -->
            "Theta" => lookup_html_unicode_num(920), // greek capital letter theta, U+0398 ISOgrk3 -->
            "Iota" => lookup_html_unicode_num(921),  // greek capital letter iota, U+0399 -->
            "Kappa" => lookup_html_unicode_num(922), // greek capital letter kappa, U+039A -->
            "Lambda" => lookup_html_unicode_num(923), // greek capital letter lambda, U+039B ISOgrk3 -->
            "Mu" => lookup_html_unicode_num(924),     // greek capital letter mu, U+039C -->
            "Nu" => lookup_html_unicode_num(925),     // greek capital letter nu, U+039D -->
            "Xi" => lookup_html_unicode_num(926),     // greek capital letter xi, U+039E ISOgrk3 -->
            "Omicron" => lookup_html_unicode_num(927), // greek capital letter omicron, U+039F -->
            "Pi" => lookup_html_unicode_num(928),     // greek capital letter pi, U+03A0 ISOgrk3 -->
            "Rho" => lookup_html_unicode_num(929),    // greek capital letter rho, U+03A1 -->
            "Sigma" => lookup_html_unicode_num(931), // greek capital letter sigma, U+03A3 ISOgrk3 -->
            "Tau" => lookup_html_unicode_num(932),   // greek capital letter tau, U+03A4 -->
            "Upsilon" => lookup_html_unicode_num(933), // greek capital letter upsilon, U+03A5 ISOgrk3 -->
            "Phi" => lookup_html_unicode_num(934), // greek capital letter phi, U+03A6 ISOgrk3 -->
            "Chi" => lookup_html_unicode_num(935), // greek capital letter chi, U+03A7 -->
            "Psi" => lookup_html_unicode_num(936), // greek capital letter psi, U+03A8 ISOgrk3 -->
            "Omega" => lookup_html_unicode_num(937), // greek capital letter omega, U+03A9 ISOgrk3 -->
            "alpha" => lookup_html_unicode_num(945), // greek small letter alpha, U+03B1 ISOgrk3 -->
            "beta" => lookup_html_unicode_num(946),  // greek small letter beta, U+03B2 ISOgrk3 -->
            "gamma" => lookup_html_unicode_num(947), // greek small letter gamma, U+03B3 ISOgrk3 -->
            "delta" => lookup_html_unicode_num(948), // greek small letter delta, U+03B4 ISOgrk3 -->
            "epsilon" => lookup_html_unicode_num(949), // greek small letter epsilon, U+03B5 ISOgrk3 -->
            "zeta" => lookup_html_unicode_num(950), // greek small letter zeta, U+03B6 ISOgrk3 -->
            "eta" => lookup_html_unicode_num(951),  // greek small letter eta, U+03B7 ISOgrk3 -->
            "theta" => lookup_html_unicode_num(952), // greek small letter theta, U+03B8 ISOgrk3 -->
            "iota" => lookup_html_unicode_num(953), // greek small letter iota, U+03B9 ISOgrk3 -->
            "kappa" => lookup_html_unicode_num(954), // greek small letter kappa, U+03BA ISOgrk3 -->
            "lambda" => lookup_html_unicode_num(955), // greek small letter lambda, U+03BB ISOgrk3 -->
            "mu" => lookup_html_unicode_num(956),     // greek small letter mu, U+03BC ISOgrk3 -->
            "nu" => lookup_html_unicode_num(957),     // greek small letter nu, U+03BD ISOgrk3 -->
            "xi" => lookup_html_unicode_num(958),     // greek small letter xi, U+03BE ISOgrk3 -->
            "omicron" => lookup_html_unicode_num(959), // greek small letter omicron, U+03BF NEW -->
            "pi" => lookup_html_unicode_num(960),     // greek small letter pi, U+03C0 ISOgrk3 -->
            "rho" => lookup_html_unicode_num(961),    // greek small letter rho, U+03C1 ISOgrk3 -->
            "sigmaf" => lookup_html_unicode_num(962), // greek small letter final sigma, U+03C2 ISOgrk3 -->
            "sigma" => lookup_html_unicode_num(963), // greek small letter sigma, U+03C3 ISOgrk3 -->
            "tau" => lookup_html_unicode_num(964),   // greek small letter tau, U+03C4 ISOgrk3 -->
            "upsilon" => lookup_html_unicode_num(965), // greek small letter upsilon, U+03C5 ISOgrk3 -->
            "phi" => lookup_html_unicode_num(966),     // greek small letter phi, U+03C6 ISOgrk3 -->
            "chi" => lookup_html_unicode_num(967),     // greek small letter chi, U+03C7 ISOgrk3 -->
            "psi" => lookup_html_unicode_num(968),     // greek small letter psi, U+03C8 ISOgrk3 -->
            "omega" => lookup_html_unicode_num(969), // greek small letter omega, U+03C9 ISOgrk3 -->
            "thetasym" => lookup_html_unicode_num(977), // greek small letter theta symbol, U+03D1 NEW -->
            "upsih" => lookup_html_unicode_num(978), // greek upsilon with hook symbol, U+03D2 NEW -->
            "piv" => lookup_html_unicode_num(982),   // greek pi symbol, U+03D6 ISOgrk3 -->
            "bull" => lookup_html_unicode_num(8226), // bullet = black small circle, U+2022 ISOpub  -->
            "hellip" => lookup_html_unicode_num(8230), // horizontal ellipsis = three dot leader, U+2026 ISOpub  -->
            "prime" => lookup_html_unicode_num(8242),  // prime = minutes = feet, U+2032 ISOtech -->
            "Prime" => lookup_html_unicode_num(8243), // double prime = seconds = inches, U+2033 ISOtech -->
            "oline" => lookup_html_unicode_num(8254), // overline = spacing overscore, U+203E NEW -->
            "frasl" => lookup_html_unicode_num(8260), // fraction slash, U+2044 NEW -->
            "weierp" => lookup_html_unicode_num(8472), // script capital P = power set = Weierstrass p, U+2118 ISOamso -->
            "image" => lookup_html_unicode_num(8465), // blackletter capital I = imaginary part, U+2111 ISOamso -->
            "real" => lookup_html_unicode_num(8476), // blackletter capital R = real part symbol, U+211C ISOamso -->
            "trade" => lookup_html_unicode_num(8482), // trade mark sign, U+2122 ISOnum -->
            "alefsym" => lookup_html_unicode_num(8501), // alef symbol = first transfinite cardinal, U+2135 NEW -->
            "larr" => lookup_html_unicode_num(8592),    // leftwards arrow, U+2190 ISOnum -->
            "uarr" => lookup_html_unicode_num(8593),    // upwards arrow, U+2191 ISOnum-->
            "rarr" => lookup_html_unicode_num(8594),    // rightwards arrow, U+2192 ISOnum -->
            "darr" => lookup_html_unicode_num(8595),    // downwards arrow, U+2193 ISOnum -->
            "harr" => lookup_html_unicode_num(8596),    // left right arrow, U+2194 ISOamsa -->
            "crarr" => lookup_html_unicode_num(8629), // downwards arrow with corner leftwards = carriage return, U+21B5 NEW -->
            "lArr" => lookup_html_unicode_num(8656),  // leftwards double arrow, U+21D0 ISOtech -->
            "uArr" => lookup_html_unicode_num(8657),  // upwards double arrow, U+21D1 ISOamsa -->
            "rArr" => lookup_html_unicode_num(8658),  // rightwards double arrow, U+21D2 ISOtech -->
            "dArr" => lookup_html_unicode_num(8659),  // downwards double arrow, U+21D3 ISOamsa -->
            "hArr" => lookup_html_unicode_num(8660),  // left right double arrow, U+21D4 ISOamsa -->
            "forall" => lookup_html_unicode_num(8704), // for all, U+2200 ISOtech -->
            "part" => lookup_html_unicode_num(8706),  // partial differential, U+2202 ISOtech  -->
            "exist" => lookup_html_unicode_num(8707), // there exists, U+2203 ISOtech -->
            "empty" => lookup_html_unicode_num(8709), // empty set = null set = diameter, U+2205 ISOamso -->
            "nabla" => lookup_html_unicode_num(8711), // nabla = backward difference, U+2207 ISOtech -->
            "isin" => lookup_html_unicode_num(8712),  // element of, U+2208 ISOtech -->
            "notin" => lookup_html_unicode_num(8713), // not an element of, U+2209 ISOtech -->
            "ni" => lookup_html_unicode_num(8715),    // contains as member, U+220B ISOtech -->
            "prod" => lookup_html_unicode_num(8719), // n-ary product = product sign, U+220F ISOamsb -->
            "sum" => lookup_html_unicode_num(8721),  // n-ary sumation, U+2211 ISOamsb -->
            "minus" => lookup_html_unicode_num(8722), // minus sign, U+2212 ISOtech -->
            "lowast" => lookup_html_unicode_num(8727), // asterisk operator, U+2217 ISOtech -->
            "radic" => lookup_html_unicode_num(8730), // square root = radical sign, U+221A ISOtech -->
            "prop" => lookup_html_unicode_num(8733),  // proportional to, U+221D ISOtech -->
            "infin" => lookup_html_unicode_num(8734), // infinity, U+221E ISOtech -->
            "ang" => lookup_html_unicode_num(8736),   // angle, U+2220 ISOamso -->
            "and" => lookup_html_unicode_num(8743),   // logical and = wedge, U+2227 ISOtech -->
            "or" => lookup_html_unicode_num(8744),    // logical or = vee, U+2228 ISOtech -->
            "cap" => lookup_html_unicode_num(8745),   // intersection = cap, U+2229 ISOtech -->
            "cup" => lookup_html_unicode_num(8746),   // union = cup, U+222A ISOtech -->
            "int" => lookup_html_unicode_num(8747),   // integral, U+222B ISOtech -->
            "there4" => lookup_html_unicode_num(8756), // therefore, U+2234 ISOtech -->
            "sim" => lookup_html_unicode_num(8764), // tilde operator = varies with = similar to, U+223C ISOtech -->
            "cong" => lookup_html_unicode_num(8773), // approximately equal to, U+2245 ISOtech -->
            "asymp" => lookup_html_unicode_num(8776), // almost equal to = asymptotic to, U+2248 ISOamsr -->
            "ne" => lookup_html_unicode_num(8800),    // not equal to, U+2260 ISOtech -->
            "equiv" => lookup_html_unicode_num(8801), // identical to, U+2261 ISOtech -->
            "le" => lookup_html_unicode_num(8804),    // less-than or equal to, U+2264 ISOtech -->
            "ge" => lookup_html_unicode_num(8805), // greater-than or equal to, U+2265 ISOtech -->
            "sub" => lookup_html_unicode_num(8834), // subset of, U+2282 ISOtech -->
            "sup" => lookup_html_unicode_num(8835), // superset of, U+2283 ISOtech -->
            "nsub" => lookup_html_unicode_num(8836), // not a subset of, U+2284 ISOamsn -->
            "sube" => lookup_html_unicode_num(8838), // subset of or equal to, U+2286 ISOtech -->
            "supe" => lookup_html_unicode_num(8839), // superset of or equal to, U+2287 ISOtech -->
            "oplus" => lookup_html_unicode_num(8853), // circled plus = direct sum, U+2295 ISOamsb -->
            "otimes" => lookup_html_unicode_num(8855), // circled times = vector product, U+2297 ISOamsb -->
            "perp" => lookup_html_unicode_num(8869), // up tack = orthogonal to = perpendicular, U+22A5 ISOtech -->
            "sdot" => lookup_html_unicode_num(8901), // dot operator, U+22C5 ISOamsb -->
            "lceil" => lookup_html_unicode_num(8968), // left ceiling = apl upstile, U+2308 ISOamsc  -->
            "rceil" => lookup_html_unicode_num(8969), // right ceiling, U+2309 ISOamsc  -->
            "lfloor" => lookup_html_unicode_num(8970), // left floor = apl downstile, U+230A ISOamsc  -->
            "rfloor" => lookup_html_unicode_num(8971), // right floor, U+230B ISOamsc  -->
            "lang" => lookup_html_unicode_num(9001), // left-pointing angle bracket = bra, U+2329 ISOtech -->
            "rang" => lookup_html_unicode_num(9002), // right-pointing angle bracket = ket, U+232A ISOtech -->
            "loz" => lookup_html_unicode_num(9674),  // lozenge, U+25CA ISOpub -->
            "spades" => lookup_html_unicode_num(9824), // black spade suit, U+2660 ISOpub -->
            "clubs" => lookup_html_unicode_num(9827), // black club suit = shamrock, U+2663 ISOpub -->
            "hearts" => lookup_html_unicode_num(9829), // black heart suit = valentine, U+2665 ISOpub -->
            "diams" => lookup_html_unicode_num(9830),  // black diamond suit, U+2666 ISOpub -->
            "quot" => lookup_html_unicode_num(34), // quotation mark = APL quote, U+0022 ISOnum -->
            "amp" => lookup_html_unicode_num(38),  // ampersand, U+0026 ISOnum -->
            "lt" => lookup_html_unicode_num(60),   // less-than sign, U+003C ISOnum -->
            "gt" => lookup_html_unicode_num(62),   // greater-than sign, U+003E ISOnum -->
            "OElig" => lookup_html_unicode_num(338), // latin capital ligature OE, U+0152 ISOlat2 -->
            "oelig" => lookup_html_unicode_num(339), // latin small ligature oe, U+0153 ISOlat2 -->
            "Scaron" => lookup_html_unicode_num(352), // latin capital letter S with caron, U+0160 ISOlat2 -->
            "scaron" => lookup_html_unicode_num(353), // latin small letter s with caron, U+0161 ISOlat2 -->
            "Yuml" => lookup_html_unicode_num(376), // latin capital letter Y with diaeresis, U+0178 ISOlat2 -->
            "circ" => lookup_html_unicode_num(710), // modifier letter circumflex accent, U+02C6 ISOpub -->
            "tilde" => lookup_html_unicode_num(732), // small tilde, U+02DC ISOdia -->
            "ensp" => lookup_html_unicode_num(8194), // en space, U+2002 ISOpub -->
            "emsp" => lookup_html_unicode_num(8195), // em space, U+2003 ISOpub -->
            "thinsp" => lookup_html_unicode_num(8201), // thin space, U+2009 ISOpub -->
            "zwnj" => lookup_html_unicode_num(8204), // zero width non-joiner, U+200C NEW RFC 2070 -->
            "zwj" => lookup_html_unicode_num(8205),  // zero width joiner, U+200D NEW RFC 2070 -->
            "lrm" => lookup_html_unicode_num(8206),  // left-to-right mark, U+200E NEW RFC 2070 -->
            "rlm" => lookup_html_unicode_num(8207),  // right-to-left mark, U+200F NEW RFC 2070 -->
            "ndash" => lookup_html_unicode_num(8211), // en dash, U+2013 ISOpub -->
            "mdash" => lookup_html_unicode_num(8212), // em dash, U+2014 ISOpub -->
            "lsquo" => lookup_html_unicode_num(8216), // left single quotation mark, U+2018 ISOnum -->
            "rsquo" => lookup_html_unicode_num(8217), // right single quotation mark, U+2019 ISOnum -->
            "sbquo" => lookup_html_unicode_num(8218), // single low-9 quotation mark, U+201A NEW -->
            "ldquo" => lookup_html_unicode_num(8220), // left double quotation mark, U+201C ISOnum -->
            "rdquo" => lookup_html_unicode_num(8221), // right double quotation mark, U+201D ISOnum -->
            "bdquo" => lookup_html_unicode_num(8222), // double low-9 quotation mark, U+201E NEW -->
            "dagger" => lookup_html_unicode_num(8224), // dagger, U+2020 ISOpub -->
            "Dagger" => lookup_html_unicode_num(8225), // double dagger, U+2021 ISOpub -->
            "permil" => lookup_html_unicode_num(8240), // per mille sign, U+2030 ISOtech -->
            "lsaquo" => lookup_html_unicode_num(8249), // single left-pointing angle quotation mark, U+2039 ISO proposed -->
            "rsaquo" => lookup_html_unicode_num(8250), // single right-pointing angle quotation mark, U+203A ISO proposed -->
            "euro" => lookup_html_unicode_num(8364),   // euro sign, U+20AC NEW -->

            _ => Err("Unknown name".to_string()),
        }
    }

    pub fn replace_html_unicode(str: &mut str) -> String {
        // Regex created based on https://en.wikipedia.org/wiki/UTF-8 and
        // https://www.w3.org/TR/html4/intro/sgmltut.html#h-3.2.3
        regex_replace_all!(
            r#"&(?:(?P<name>[a-zA-Z]+)|(?P<code>#(?:(?P<code_num>\d+)|(?P<code_hex>x[0-9a-fA-F]+))));"#,
            str,
            |_, name: &str, _, code_num: &str, code_hex: &str| {
                if !name.is_empty() {
                    return lookup_html_unicode_name(name).unwrap();
                }

                if !code_num.is_empty() {
                    return lookup_html_unicode_num(code_num.parse::<u32>().unwrap()).unwrap();
                }

                if !code_hex.is_empty() {
                    return lookup_html_unicode_num(
                        u32::from_str_radix(&code_hex[1..], 16).unwrap(),
                    )
                    .unwrap();
                }

                return "".to_string();
            }
        ).to_string()
    }
}
