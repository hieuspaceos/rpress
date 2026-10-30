//! Port of `tourmaster_tour_options` (tour/include/tour-option.php ~lines 188-1193)
//! plus the `tourmaster-review-option` page block (lines 1196-1226), whose two tabs
//! (`manage-review`, `add-a-review`) are merged here so `TABS` has all 7 tabs.
//!
//! Deviations from the shared contract (documented for elements.rs/save.rs):
//! - `options` stays `&[(&str, &str)]` for literal option lists. PHP string options
//!   (`'sidebar-default'`, `'thumbnail-size'`, `'post_type'`, `'skin'`) go into the
//!   new `options_source`; `'options-data'` (e.g. `tour_service`) into `options_data`.
//! - `button` fields add `button_title`, `data_type`, `data_action`, `data_fields`
//!   (PHP keys `button-title`, `data-type`, `data-action`, `data-fields`).
//! - `tab_children` exists per contract but is always `None`: the PHP nested `tabs`
//!   items keep their per-item fields in `options`, mapped here to `children`.
//! - `wrapper_class` covers both PHP spellings (`wrapper-class` top level,
//!   `wrapper_class` inside tab items — the JS template reads the underscored key).
//! - `condition` is the raw `json_encode` output of the PHP array, e.g.
//!   `{"header-image":["feature-image","custom-image"]}`.
//! - `ical-description` description contains the `{ical_url}` placeholder where PHP
//!   concatenated `$ical_url`; renderer substitutes it.
//! - radioimage URLs: `TOURMASTER_URL . '/images/option/x.jpg'` ->
//!   `/admin/wp/tourmaster/images/option/x.jpg`.

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Field {
    pub slug: &'static str,
    pub title: &'static str,
    pub ftype: &'static str,
    pub item_type: Option<&'static str>,
    pub options: &'static [(&'static str, &'static str)],
    pub options_source: Option<&'static str>,
    pub options_data: Option<&'static str>,
    pub children: &'static [Field],
    pub tab_children: Option<&'static [(&'static str, &'static [Field])]>,
    pub single: Option<&'static str>,
    pub condition: Option<&'static str>,
    pub wrapper_class: &'static str,
    pub description: &'static str,
    pub default: &'static str,
    pub settings: &'static [(&'static str, &'static str)],
    pub title_color: &'static str,
    pub column_size: u8,
    pub bottom_divider: bool,
    pub right_divider: &'static str,
    pub clear: bool,
    pub data_input_type: &'static str,
    pub button_title: &'static str,
    pub data_type: &'static str,
    pub data_action: &'static str,
    pub data_fields: &'static [&'static str],
}

#[allow(dead_code)]
pub struct Tab {
    pub slug: &'static str,
    pub title: &'static str,
    pub fields: &'static [Field],
}

const F: Field = Field {
    slug: "",
    title: "",
    ftype: "",
    item_type: None,
    options: &[],
    options_source: None,
    options_data: None,
    children: &[],
    tab_children: None,
    single: None,
    condition: None,
    wrapper_class: "",
    description: "",
    default: "",
    settings: &[],
    title_color: "",
    column_size: 0,
    bottom_divider: false,
    right_divider: "",
    clear: false,
    data_input_type: "",
    button_title: "",
    data_type: "",
    data_action: "",
    data_fields: &[],
};

static HEADER_IMAGE_OPTIONS: &[(&str, &str)] = &[
    ("feature-image", "Feature Image"),
    ("custom-image", "Custom Image"),
    ("slider", "Slider"),
    ("gallery", "Gallery"),
    ("video", "Video ( Youtube & Vimeo )"),
    ("html5-video", "Html5 Video"),
    ("revolution-slider", "Revolution Slider"),
];

// ---------------------------------------------------------------------------
// Tab: general
// ---------------------------------------------------------------------------

static GENERAL: &[Field] = &[
    Field {
        slug: "header-image",
        title: "Header Background",
        ftype: "combobox",
        options: HEADER_IMAGE_OPTIONS,
        ..F
    },
    Field {
        slug: "header-image-custom",
        title: "Header Custom Image",
        ftype: "upload",
        condition: Some(r#"{"header-image":"custom-image"}"#),
        ..F
    },
    Field {
        slug: "header-revolution-slider-id",
        title: "Header Revolution Slider ID",
        ftype: "text",
        condition: Some(r#"{"header-image":"revolution-slider"}"#),
        ..F
    },
    Field {
        slug: "header-top-padding",
        title: "Header Top Padding",
        ftype: "text",
        data_input_type: "pixel",
        condition: Some(r#"{"header-image":["feature-image","custom-image"]}"#),
        description: "Leaving this field blank to use default value from tourmaster option",
        ..F
    },
    Field {
        slug: "header-bottom-padding",
        title: "Header Bottom Padding",
        ftype: "text",
        data_input_type: "pixel",
        condition: Some(r#"{"header-image":["feature-image","custom-image"]}"#),
        description: "Leaving this field blank to use default value from tourmaster option",
        ..F
    },
    Field {
        slug: "header-background-overlay-opacity",
        title: "Title Background Overlay Opacity",
        ftype: "text",
        description: "Fill the number between 0 - 1 ( Leave Blank For Default Value )",
        ..F
    },
    Field {
        slug: "header-background-gradient",
        title: "Title Background Gradient",
        ftype: "combobox",
        options: &[
            ("default", "Default"),
            ("both", "Both"),
            ("top", "Top"),
            ("bottom", "Bottom"),
            ("none", "None"),
        ],
        ..F
    },
    Field {
        slug: "header-slider",
        title: "Slider Images",
        ftype: "custom",
        item_type: Some("gallery"),
        condition: Some(r#"{"header-image":["slider","gallery"]}"#),
        wrapper_class: "tourmaster-fullsize",
        ..F
    },
    Field {
        slug: "lightbox-video-url",
        title: "Lightbox Video URL",
        ftype: "text",
        condition: Some(r#"{"header-image":"gallery"}"#),
        ..F
    },
    Field {
        slug: "header-slider-thumbnail",
        title: "Slider Images Thumbnail Size",
        ftype: "combobox",
        options_source: Some("thumbnail-size"),
        condition: Some(r#"{"header-image":"slider"}"#),
        ..F
    },
    Field {
        slug: "background-video-url",
        title: "Background Video URL",
        ftype: "text",
        condition: Some(r#"{"header-image":"video"}"#),
        ..F
    },
    Field {
        slug: "background-video-url-mp4",
        title: "Background Video URL (MP4)",
        ftype: "text",
        condition: Some(r#"{"header-image":"html5-video"}"#),
        ..F
    },
    Field {
        slug: "background-video-url-webm",
        title: "Background Video URL (WEBM)",
        ftype: "text",
        condition: Some(r#"{"header-image":"html5-video"}"#),
        ..F
    },
    Field {
        slug: "background-video-url-ogg",
        title: "Background Video URL (ogg)",
        ftype: "text",
        condition: Some(r#"{"header-image":"html5-video"}"#),
        ..F
    },
    Field {
        slug: "background-video-image",
        title: "Background Image Fallback",
        ftype: "upload",
        condition: Some(r#"{"header-image":["video","html5-video"]}"#),
        description: "This background will be showing up when the device you're using cannot render the video as background ( eg. mobile device )",
        ..F
    },
    Field {
        slug: "show-wordpress-editor-content",
        title: "Show Wordpress Editor Content",
        ftype: "checkbox",
        default: "enable",
        ..F
    },
    Field {
        slug: "enable-page-title",
        title: "Enable Page Title",
        ftype: "checkbox",
        default: "enable",
        ..F
    },
    Field {
        slug: "enable-header-review-number",
        title: "Enable Header Review Number",
        ftype: "checkbox",
        default: "enable",
        ..F
    },
    Field {
        slug: "thumbnail-link",
        title: "Thumbnail Link",
        ftype: "combobox",
        single: Some("tourmaster-thumbnail-link"),
        options: &[
            ("single-tour", "Single Tour"),
            ("lightbox-to-video", "Lightbox To Video"),
        ],
        ..F
    },
    Field {
        slug: "thumbnail-video-url",
        title: "Thumbnail Video Url",
        ftype: "text",
        single: Some("tourmaster-thumbnail-video-url"),
        condition: Some(r#"{"thumbnail-link":"lightbox-to-video"}"#),
        ..F
    },
    Field {
        slug: "promo-text",
        title: "Promo Text",
        ftype: "text",
        ..F
    },
    Field {
        slug: "promo-text-ribbon-text-color",
        title: "Promo Text Ribbon Text Color",
        ftype: "colorpicker",
        default: "#ffffff",
        ..F
    },
    Field {
        slug: "promo-text-ribbon-background",
        title: "Promo Text Ribbon Background",
        ftype: "colorpicker",
        default: "#467be7",
        ..F
    },
    Field {
        slug: "sidebar-widget",
        title: "Sidebar Widget",
        ftype: "combobox",
        options_source: Some("sidebar-default"),
        default: "default",
        ..F
    },
];

// ---------------------------------------------------------------------------
// Tab: tour-settings
// ---------------------------------------------------------------------------

static TOUR_SETTINGS: &[Field] = &[
    Field {
        slug: "ical-description",
        ftype: "description",
        description: r#"ICal URL : <a href="{ical_url}" target="_blank" >{ical_url}</a>"#,
        ..F
    },
    Field {
        slug: "enable-payment",
        title: "Enable Payment",
        ftype: "combobox",
        options: &[
            ("", "Default"),
            ("enable", "Enable"),
            ("disable", "Disable"),
        ],
        ..F
    },
    Field {
        slug: "payment-admin-approval",
        title: "Needs Admin Approval Before Payment",
        ftype: "combobox",
        options: &[
            ("", "Default"),
            ("enable", "Enable"),
            ("disable", "Disable"),
        ],
        ..F
    },
    Field {
        slug: "form-settings",
        title: "Reservation Bar",
        ftype: "combobox",
        options: &[
            ("booking", "Only Booking Form"),
            ("enquiry", "Only Enquiry Form"),
            ("both", "Both Booking & Enquiry Form"),
            ("custom", "Custom Code"),
            ("none", "None ( Hide the right side out )"),
        ],
        default: "booking",
        ..F
    },
    Field {
        slug: "booking-bar-custom-title",
        title: "Custom Title",
        ftype: "text",
        condition: Some(r#"{"form-settings":["booking","enquiry","both"]}"#),
        description: "Only apply to tour style 3",
        ..F
    },
    Field {
        slug: "extra-booking-info",
        title: "Custom Extra Booking Info",
        ftype: "textarea",
        single: Some("tourmaster-extra-booking-info"),
        description: r#"You can see how to create the fields <a href="http://support.goodlayers.com/document/2017/10/06/tourmaster-modifying-the-enquiry-form/" target="_blank" >here</a><br>Use for gathering plan data only. This custom booking info has nothing to do with system calculation such as booking date"#,
        condition: Some(r#"{"form-settings":["booking","both"]}"#),
        ..F
    },
    Field {
        slug: "contact-detail-fields",
        title: "Custom Contact Detail Fields",
        ftype: "textarea",
        single: Some("tourmaster-contact-detail-fields"),
        description: r#"You can see how to create the fields <a href="https://support.goodlayers.com/document/2018/05/01/tourmaster-modifying-the-contact-detail-fields-since-v3-0-8/" target="_blank" >here</a><br>If left blank, the system will use default settings from Tour Master panel settings."#,
        condition: Some(r#"{"form-settings":["booking","both"]}"#),
        ..F
    },
    Field {
        slug: "enquiry-form-fields",
        title: "Custom Enquiry Form Fields",
        ftype: "textarea",
        single: Some("tourmaster-enquiry-form-fields"),
        description: r#"You can see how to create the fields <a href="https://support.goodlayers.com/document/2017/10/06/tourmaster-modifying-the-enquiry-form/" target="_blank" >HERE</a>"#,
        condition: Some(r#"{"form-settings":["enquiry","both"]}"#),
        ..F
    },
    Field {
        slug: "enquiry-form-mail-content-admin",
        title: "Custom Enquiry Form Mail Content (Admin)",
        ftype: "textarea",
        single: Some("tourmaster-enquiry-form-mail-content-admin"),
        condition: Some(r#"{"form-settings":["enquiry","both"]}"#),
        ..F
    },
    Field {
        slug: "enquiry-form-mail-content-customer",
        title: "Custom Enquiry Form Mail Content (Customer)",
        ftype: "textarea",
        single: Some("tourmaster-enquiry-form-mail-content-customer"),
        condition: Some(r#"{"form-settings":["enquiry","both"]}"#),
        ..F
    },
    Field {
        slug: "form-custom-title",
        title: "Custom Code Title",
        ftype: "text",
        condition: Some(r#"{"form-settings":"custom"}"#),
        description: "Leave this field blank to display header price",
        ..F
    },
    Field {
        slug: "form-custom-code",
        title: "Custom Code",
        ftype: "textarea",
        condition: Some(r#"{"form-settings":"custom"}"#),
        ..F
    },
    Field {
        slug: "show-price",
        title: "Show Header Price",
        ftype: "checkbox",
        default: "enable",
        condition: Some(r#"{"form-settings":"enquiry"}"#),
        ..F
    },
    Field {
        slug: "date-selection-type",
        title: "Date Selection Type",
        ftype: "combobox",
        options: &[
            ("calendar", "Calendar"),
            ("date-list", "Date List"),
        ],
        condition: Some(r#"{"form-settings":["booking","both"]}"#),
        ..F
    },
    Field {
        slug: "last-minute-booking",
        title: "Last Minute Booking (Hour)",
        ftype: "text",
        condition: Some(r#"{"form-settings":["booking","both"]}"#),
        description: "Specify the number of hours prior to the travel time you want to close the booking system.",
        ..F
    },
    Field {
        slug: "book-in-advance",
        title: "Book In Advance (Month)",
        ftype: "text",
        single: Some("tourmaster-book-in-advance"),
        condition: Some(r#"{"form-settings":["booking","both"]}"#),
        description: "For example, If you fill the number \"10\" (for ten months) and today is in March 2019, customers will have an ability to book the tour from today until Jan 2020 (ten months from current month). Leave this field blank for unlimited booking in advanced.",
        ..F
    },
    Field {
        slug: "deposit-booking",
        title: "Deposit Booking",
        ftype: "combobox",
        options: &[
            ("default", "Default"),
            ("enable", "Enable (Custom)"),
            ("disable", "Disable"),
        ],
        description: "Default value can be set at the \"Tourmaster\" plugin option.",
        condition: Some(r#"{"form-settings":["booking","both"]}"#),
        ..F
    },
    Field {
        slug: "deposit-amount",
        title: "Deposit Amount (%)",
        ftype: "text",
        description: "Only fill number here.",
        condition: Some(r#"{"form-settings":["booking","both"],"deposit-booking":"enable"}"#),
        ..F
    },
    Field {
        slug: "deposit2-amount",
        title: "Deposit 2 Amount (%)",
        ftype: "text",
        description: "Only fill number here.",
        condition: Some(r#"{"form-settings":["booking","both"],"deposit-booking":"enable"}"#),
        ..F
    },
    Field {
        slug: "deposit3-amount",
        title: "Deposit 3 Amount (%)",
        ftype: "text",
        description: "Only fill number here.",
        condition: Some(r#"{"form-settings":["booking","both"],"deposit-booking":"enable"}"#),
        ..F
    },
    Field {
        slug: "deposit4-amount",
        title: "Deposit 4 Amount (%)",
        ftype: "text",
        description: "Only fill number here.",
        condition: Some(r#"{"form-settings":["booking","both"],"deposit-booking":"enable"}"#),
        ..F
    },
    Field {
        slug: "deposit5-amount",
        title: "Deposit 5 Amount (%)",
        ftype: "text",
        description: "Only fill number here.",
        condition: Some(r#"{"form-settings":["booking","both"],"deposit-booking":"enable"}"#),
        ..F
    },
    Field {
        slug: "tour-price-text",
        title: "Tour Price Text",
        ftype: "text",
        description: "Use for search function and displaying as tour information. Only fill number here.",
        condition: Some(r#"{"form-settings":["booking","enquiry","both","custom"]}"#),
        ..F
    },
    Field {
        slug: "tour-price-discount-text",
        title: "Tour Price Discount Text",
        ftype: "text",
        description: "Use for search function and displaying as tour information. Only fill number here.",
        condition: Some(r#"{"form-settings":["booking","enquiry","both","custom"]}"#),
        ..F
    },
    Field {
        slug: "tour-price-range",
        title: "Tour Price Range ( Only For Schema Data )",
        ftype: "text",
        description: "This is an example of price renge format \"$100 - $1000\"",
        single: Some("tourmaster-tour-price-range"),
        condition: Some(r#"{"form-settings":["booking","enquiry","both","custom"]}"#),
        ..F
    },
    Field {
        slug: "duration-text",
        title: "Duration Text",
        ftype: "text",
        description: "Only for displaying as tour information.",
        ..F
    },
    Field {
        slug: "multiple-duration",
        title: "Duration (Days)",
        ftype: "text",
        description: "Ex. Fill \"3\" for three days (Only Number is Allowed)",
        ..F
    },
    Field {
        slug: "date-range",
        title: "Date Range (Availability)",
        ftype: "text",
        description: "Only for displaying as tour information.",
        ..F
    },
    Field {
        slug: "departure-location",
        title: "Departure Location",
        ftype: "text",
        description: "Only for displaying as tour information.",
        ..F
    },
    Field {
        slug: "return-location",
        title: "Destination Location",
        ftype: "text",
        description: "Only for displaying as tour information.",
        ..F
    },
    Field {
        slug: "minimum-age",
        title: "Minimum Age",
        ftype: "text",
        description: "Only for displaying as tour information. Ex. \"16+\"",
        ..F
    },
    Field {
        slug: "minimum-people-per-booking",
        title: "Minimum People Per Booking",
        ftype: "text",
        single: Some("tourmaster-min-people-per-booking"),
        description: "This is a global value for every tour. However, you can assign different value in each tour as well.",
        condition: Some(r#"{"form-settings":["booking","both"]}"#),
        ..F
    },
    Field {
        slug: "maximum-people-per-booking",
        title: "Maximum People Per Booking",
        ftype: "text",
        single: Some("tourmaster-max-people-per-booking"),
        condition: Some(r#"{"form-settings":["booking","both"]}"#),
        ..F
    },
    Field {
        slug: "maximum-people",
        title: "Maximum People",
        ftype: "text",
        single: Some("tourmaster-max-people"),
        condition: Some(r#"{"form-settings":["booking","both"]}"#),
        description: "Only for displaying as tour information.",
        ..F
    },
    Field {
        slug: "display-single-tour-info",
        title: "Display Single Tour Info",
        ftype: "checkbox",
        default: "enable",
        ..F
    },
    Field {
        slug: "require-each-traveller-info",
        title: "Require Each Traveller's Info",
        ftype: "checkbox",
        default: "enable",
        condition: Some(r#"{"form-settings":["booking","both"]}"#),
        description: "This option requires customer to fill name and last name of each traveller.",
        ..F
    },
    Field {
        slug: "additional-traveller-fields",
        title: "Custom Traveller Detail Fields",
        ftype: "textarea",
        description: r#"You can see how to create the fields <a href="https://support.goodlayers.com/document/2018/05/03/tourmaster-modifying-the-traveller-detail-fields-since-v3-0-8/" target="_blank" >HERE</a>"#,
        condition: Some(r#"{"form-settings":["booking","both"],"require-each-traveller-info":"enable"}"#),
        ..F
    },
    Field {
        slug: "require-traveller-info-title",
        title: "Require Traveller's Title (Mr/Mrs)",
        ftype: "checkbox",
        condition: Some(r#"{"require-each-traveller-info":"enable","form-settings":["booking","both"]}"#),
        default: "enable",
        ..F
    },
    Field {
        slug: "require-traveller-passport",
        title: "Require Traveller's Passport",
        ftype: "checkbox",
        condition: Some(r#"{"require-each-traveller-info":"enable","form-settings":["booking","both"]}"#),
        default: "disable",
        ..F
    },
    Field {
        slug: "tour-service",
        title: "Tour Service",
        ftype: "multi-combobox",
        options_source: Some("post_type"),
        options_data: Some("tour_service"),
        condition: Some(r#"{"form-settings":["booking","both"]}"#),
        ..F
    },
    Field {
        slug: "custom-excerpt",
        title: "Custom Excerpt",
        ftype: "textarea",
        ..F
    },
    Field {
        slug: "link-proceed-booking-to-external-url",
        title: "Link Proceed Booking Button To External URL",
        ftype: "text",
        condition: Some(r#"{"form-settings":["booking","both"]}"#),
        description: "This option will ignore all booking variables.",
        ..F
    },
    Field {
        slug: "external-url-text",
        title: "External URL Text",
        ftype: "textarea",
        condition: Some(r#"{"form-settings":["booking","both"]}"#),
        description: "Only works with external url.",
        ..F
    },
    Field {
        slug: "enable-review",
        title: "Enable Review",
        ftype: "checkbox",
        default: "enable",
        ..F
    },
];

// ---------------------------------------------------------------------------
// Tab: date-price
// ---------------------------------------------------------------------------

static PACKAGE_CHILDREN: &[Field] = &[
    Field {
        slug: "default-package",
        title: "Default Package",
        ftype: "checkbox",
        default: "disable",
        description: "Enable to pre-selected this package on page load. Only the first package that enable this option is effected.",
        ..F
    },
    Field {
        slug: "group-slug",
        title: "Package Group Alias",
        ftype: "text",
        description: "Fill the same string on each package to group the people from 2 package together ( for Max people option ).",
        ..F
    },
    Field {
        slug: "title",
        title: "Package Title",
        ftype: "text",
        description: "Each package title has to be in different name.",
        ..F
    },
    Field {
        slug: "caption",
        title: "Package Caption",
        ftype: "text",
        ..F
    },
    Field {
        slug: "start-time",
        title: "Start Time",
        ftype: "time",
        ..F
    },
    Field {
        slug: "base-price-title",
        title: "BASE PRICE",
        ftype: "title",
        ..F
    },
    Field {
        slug: "person-price",
        title: "Price Per Person",
        ftype: "text",
        description: "Price per person (Fill only number).",
        ..F
    },
    Field {
        slug: "enable-supplement-pricing",
        title: "Enable Supplement Pricing",
        ftype: "checkbox",
        default: "disable",
        ..F
    },
    Field {
        slug: "single-supplement-price",
        title: "Single Supplement Price",
        ftype: "text",
        ..F
    },
    Field {
        slug: "triple-supplement-price",
        title: "Triple Supplement Price",
        ftype: "text",
        ..F
    },
    Field {
        slug: "adult-price",
        title: "Adult",
        ftype: "text",
        description: "Price per person (Fill only number). * Leave this field blank to not apply.",
        ..F
    },
    Field {
        slug: "male-price",
        title: "Male",
        ftype: "text",
        description: "Price per person (Fill only number). * Leave this field blank to not apply.",
        ..F
    },
    Field {
        slug: "female-price",
        title: "Female",
        ftype: "text",
        description: "Price per person (Fill only number). * Leave this field blank to not apply.",
        ..F
    },
    Field {
        slug: "children-price",
        title: "Child",
        ftype: "text",
        description: "Price per person (Fill only number). * Leave this field blank to not apply.",
        ..F
    },
    Field {
        slug: "student-price",
        title: "Student",
        ftype: "text",
        description: "Price per person (Fill only number). * Leave this field blank to not apply.",
        ..F
    },
    Field {
        slug: "infant-price",
        title: "Infant",
        ftype: "text",
        description: "Price per person (Fill only number). * Leave this field blank to not apply.",
        ..F
    },
    Field {
        slug: "group-price",
        title: "Price per group",
        ftype: "text",
        description: "Fill only number.",
        ..F
    },
    Field {
        slug: "max-group",
        title: "Max group",
        ftype: "text",
        description: "How many group you can accept in this tour? (Fill only number).",
        ..F
    },
    Field {
        slug: "max-group-people",
        title: "Max people for each group",
        ftype: "text",
        description: "People amount in each group (Fill only number).",
        ..F
    },
    Field {
        slug: "same-gender",
        title: "Same Gender Required",
        ftype: "checkbox",
        default: "disable",
        description: "This feature will allow only one gender in the this package. Ex. If female book first, the rest has to be female as well. However, mix gender will be allowed if women and men book at the same time by the same customer.",
        ..F
    },
    Field {
        slug: "room-base-price-title",
        title: "ROOM BASED PRICE",
        ftype: "title",
        ..F
    },
    Field {
        slug: "initial-price",
        title: "Initial Price",
        ftype: "text",
        description: "This price based on 2 adults",
        ..F
    },
    Field {
        slug: "single-discount",
        title: "Single Discount",
        ftype: "text",
        description: "This discount will be used for deducting the price of Initial Price. Ex, If you set Initial Price as $100 and Single Discount as $30. If there’re two guests in this room, they will pay for $100. However, if there's only one guest in this room, he/she will pay for only $70 instead of $100. This option is an alternative for single supplement.",
        ..F
    },
    Field {
        slug: "additional-person",
        title: "Additional Person",
        ftype: "text",
        description: "Price per person (Fill only number).",
        ..F
    },
    Field {
        slug: "additional-adult",
        title: "Additional Adult",
        ftype: "text",
        description: "Price per person (Fill only number). * Leave this field blank to not apply.",
        ..F
    },
    Field {
        slug: "additional-male",
        title: "Additional Male",
        ftype: "text",
        description: "Price per person (Fill only number). * Leave this field blank to not apply.",
        ..F
    },
    Field {
        slug: "additional-female",
        title: "Additional Female",
        ftype: "text",
        description: "Price per person (Fill only number). * Leave this field blank to not apply.",
        ..F
    },
    Field {
        slug: "additional-children",
        title: "Additional Child",
        ftype: "text",
        description: "Price per person (Fill only number). * Leave this field blank to not apply.",
        ..F
    },
    Field {
        slug: "additional-student",
        title: "Additional Student",
        ftype: "text",
        description: "Price per person (Fill only number). * Leave this field blank to not apply.",
        ..F
    },
    Field {
        slug: "additional-infant",
        title: "Additional Infant",
        ftype: "text",
        description: "Price per person (Fill only number). * Leave this field blank to not apply.",
        ..F
    },
    Field {
        slug: "minimum-people-per-booking",
        title: "Minimum People Per Booking",
        ftype: "text",
        ..F
    },
    Field {
        slug: "maximum-people-per-booking",
        title: "Maximum People Per Booking",
        ftype: "text",
        ..F
    },
    Field {
        slug: "max-room",
        title: "Max Room",
        ftype: "text",
        ..F
    },
    Field {
        slug: "max-people-per-room",
        title: "Max People Per Room",
        ftype: "text",
        ..F
    },
    Field {
        slug: "max-people",
        title: "Maximum People",
        ftype: "text",
        ..F
    },
];

static DATE_PRICE_CHILDREN: &[Field] = &[
    Field {
        slug: "date",
        title: "Date",
        ftype: "datepicker",
        wrapper_class: "tourmaster-small-title",
        ..F
    },
    Field {
        slug: "day",
        title: "Day",
        ftype: "checkboxes",
        options: &[
            ("monday", "Mon"),
            ("tuesday", "Tue"),
            ("wednesday", "Wed"),
            ("thursday", "Thu"),
            ("friday", "Fri"),
            ("saturday", "Sat"),
            ("sunday", "Sun"),
            ("select-all", "Select All"),
            ("deselect-all", "Deselect All"),
        ],
        ..F
    },
    Field {
        slug: "month",
        title: "Month",
        ftype: "checkboxes",
        options: &[
            ("1", "Jan"),
            ("2", "Feb"),
            ("3", "Mar"),
            ("4", "Apr"),
            ("5", "May"),
            ("6", "Jun"),
            ("7", "Jul"),
            ("8", "Aug"),
            ("9", "Sep"),
            ("10", "Oct"),
            ("11", "Nov"),
            ("12", "Dec"),
            ("select-all", "Select All"),
            ("deselect-all", "Deselect All"),
        ],
        ..F
    },
    Field {
        slug: "year",
        title: "Year",
        ftype: "checkboxes",
        options: &[
            ("2019", "2019"),
            ("2020", "2020"),
            ("2021", "2021"),
            ("2022", "2022"),
            ("2023", "2023"),
            ("2024", "2024"),
            ("2025", "2025"),
            ("2026", "2026"),
        ],
        ..F
    },
    Field {
        slug: "extra-date-description",
        ftype: "description",
        description: "Fill the date in yyyy-mm-dd format and separated the date using comma. Eg. 2020-12-25,2020-12-26,2020-12-27",
        ..F
    },
    Field {
        slug: "extra-date",
        title: "INCLUDE EXTRA DATES USING DATE FORMAT",
        ftype: "textarea",
        wrapper_class: "tourmaster-full-size",
        title_color: "#67b1a1",
        ..F
    },
    Field {
        slug: "exclude-extra-date",
        title: "EXCLUDE EXTRA DATES USING DATE FORMAT",
        ftype: "textarea",
        wrapper_class: "tourmaster-full-size",
        title_color: "#be7272",
        ..F
    },
    Field {
        slug: "pricing-title",
        title: "PRICING",
        ftype: "title",
        wrapper_class: "tourmaster-middle-with-divider",
        ..F
    },
    Field {
        slug: "pricing-method",
        title: "Pricing Method",
        ftype: "radioimage",
        options: &[
            ("fixed", "/admin/wp/tourmaster/images/option/fixed-price.jpg"),
            ("variable", "/admin/wp/tourmaster/images/option/variable-price.jpg"),
            ("group", "/admin/wp/tourmaster/images/option/group-price.jpg"),
        ],
        description: "* Variable pricing will differentiate the price of adult, children, student and infant.",
        ..F
    },
    Field {
        slug: "pricing-room-base",
        title: "Enable Room Base",
        ftype: "radioimage",
        options: &[
            ("enable", "/admin/wp/tourmaster/images/option/room-base-enable.jpg"),
            ("disable", "/admin/wp/tourmaster/images/option/room-base-disable.jpg"),
        ],
        condition: Some(r#"{"pricing-method":["fixed","variable"]}"#),
        description: "* Calculate tour price based on the hotel's room. For example, 2 Adults 2 Rooms will be more expensive than 2 Adults 1 Room.",
        ..F
    },
    Field {
        slug: "base-price-description",
        ftype: "description",
        description: "When you choose <strong>Room Base Pricing</strong>. There will be 2 parts of the price. The final price will be the summary of these two.",
        condition: Some(r#"{"pricing-method":["fixed","variable"]}"#),
        ..F
    },
    Field {
        slug: "package",
        title: "Add Package",
        ftype: "tabs",
        settings: &[("tab-title", "Default Package")],
        children: PACKAGE_CHILDREN,
        condition: Some(r#"{"pricing-method":"css-condition","pricing-room-base":"css-condition"}"#),
        ..F
    },
    Field {
        slug: "select-package-text",
        title: "Select a Package Text",
        ftype: "text",
        description: "Leave blank for default",
        ..F
    },
];

static GROUP_DISCOUNT_CHILDREN: &[Field] = &[
    Field {
        slug: "traveller-number",
        title: "Travellers number",
        ftype: "text",
        ..F
    },
    Field {
        slug: "discount",
        title: "Discount",
        ftype: "text",
        ..F
    },
    Field {
        slug: "description",
        ftype: "description",
        description: "* Fill only number for fixed amount, ex. '10' for $10. Fill % at the end if using as percentage, ex. '10%'",
        ..F
    },
];

static GROUP_DISCOUNT_CATEGORY_OPTIONS: &[(&str, &str)] = &[
    ("adult", "Adult"),
    ("male", "Male"),
    ("female", "Female"),
    ("children", "Children"),
    ("student", "Student"),
    ("infant", "Infant"),
];

static DATE_PRICE: &[Field] = &[
    Field {
        slug: "column-1-open",
        ftype: "column",
        column_size: 30,
        right_divider: "fa fa-angle-right",
        bottom_divider: true,
        ..F
    },
    Field {
        slug: "tour-type",
        title: "1. Select Tour Type",
        ftype: "radioimage",
        options: &[
            ("single", "/admin/wp/tourmaster/images/option/type-one.jpg"),
            ("multiple", "/admin/wp/tourmaster/images/option/type-multiple.jpg"),
        ],
        wrapper_class: "tourmaster-center-option",
        ..F
    },
    Field {
        slug: "column-1-close",
        ftype: "column-close",
        ..F
    },
    Field {
        slug: "column-2-open",
        ftype: "column",
        column_size: 30,
        bottom_divider: true,
        ..F
    },
    Field {
        slug: "tour-timing-method",
        title: "2. Select Timing Method",
        ftype: "radioimage",
        options: &[
            ("single", "/admin/wp/tourmaster/images/option/timing-one.jpg"),
            ("recurring", "/admin/wp/tourmaster/images/option/timing-recurring.jpg"),
        ],
        wrapper_class: "tourmaster-center-option",
        ..F
    },
    Field {
        slug: "column-2-close",
        ftype: "column-close",
        clear: true,
        ..F
    },
    Field {
        slug: "date-price",
        title: "Add Date & Price",
        ftype: "custom",
        item_type: Some("tabs"),
        children: DATE_PRICE_CHILDREN,
        settings: &[
            ("tab-title", r#"Date<i class="fa fa-edit" ></i>"#),
            ("allow-duplicate", r#"<i class="fa fa-copy" ></i>Duplicate"#),
        ],
        condition: Some(r#"{"tour-type":"css-condition","tour-timing-method":"css-condition"}"#),
        wrapper_class: "tourmaster-with-bottom-divider",
        ..F
    },
    Field {
        slug: "group-discount-title",
        title: "Group Discount",
        ftype: "title",
        wrapper_class: "tourmaster-main-title",
        ..F
    },
    Field {
        slug: "group-discount-category",
        title: "Group Discount Category Counting",
        ftype: "multi-combobox",
        options: GROUP_DISCOUNT_CATEGORY_OPTIONS,
        description: "Leave this field blank to select all traveller types. Use \"ctrl\" to select multiple or deselect the option.<br><br>This option will let you choose which group to be counted for discount. Ex. if you choose to use only Adult to be counted and choose 3 Travellers Number to get discount. When select 2 adult + 1 child, this condition will not be met. However, if select 3 adults + 1 child, this condition met. Note that this option apply to Variable Price only.",
        ..F
    },
    Field {
        slug: "group-discount-apply",
        title: "Group Discount Apply To",
        ftype: "multi-combobox",
        options: GROUP_DISCOUNT_CATEGORY_OPTIONS,
        description: "You can choose  which category to get discount when discount condition met.",
        ..F
    },
    Field {
        slug: "group-discount-per-person",
        title: "Group Discount Based On Person",
        ftype: "checkbox",
        default: "disable",
        description: "This option will be automatically set to \"Enable\" if the \"Group Discount Apply To\" option is selected.<br><br>If you turn this option on, the discount will apply on per person basis and if you're using 'Room Base' pricing, it will only apply to 'Base Price' and won't apply to 'Room Based Price'. Please also note that with this option, the discount won't be applied to \"Tour Service\" as well. However, if you turn this option off, the discount will be applied to everything and will be shown as discount at the end of price breakdown.",
        ..F
    },
    Field {
        slug: "group-discount",
        title: "Add Group Discount",
        ftype: "custom",
        item_type: Some("group-discount"),
        children: GROUP_DISCOUNT_CHILDREN,
        description: "For example, if you create 2 discount boxes, and for the first box, you set up 5 travellers with 15% discount and for another box, 10 travellers with 25% discount. When customers book for 5,6,7,8,9 travellers, they will get 15% off. However, if they book for 10, 11, 12 ( and so on ) travellers, they will get 25% off.",
        ..F
    },
];

// ---------------------------------------------------------------------------
// Tab: urgency-message
// ---------------------------------------------------------------------------

static URGENCY_MESSAGE: &[Field] = &[
    Field {
        slug: "enable-urgency-message",
        title: "Enable Urgency Message :",
        ftype: "checkbox",
        default: "disable",
        description: "By enabling this option, the urgent message will be shown in the front-end of the single tour. Ex. \"20 travellers are considering this tour right now!\"<br>** Urgency message will be disappeared for 1 day after you close it.",
        ..F
    },
    Field {
        slug: "real-urgency-message",
        title: "Use Real Data :",
        ftype: "checkbox",
        default: "enable",
        description: "Real data will record each user for 1 hour.",
        ..F
    },
    Field {
        slug: "urgency-message-number-from",
        title: "Number From :",
        ftype: "text",
        default: "5",
        condition: Some(r#"{"real-urgency-message":"disable"}"#),
        description: "The system will randomly pick the number between \"from\" and \"to\" fields.",
        ..F
    },
    Field {
        slug: "urgency-message-number-to",
        title: "Number To :",
        ftype: "text",
        default: "10",
        condition: Some(r#"{"real-urgency-message":"disable"}"#),
        ..F
    },
];

// ---------------------------------------------------------------------------
// Tab: group-message (Reminder & Message)
// ---------------------------------------------------------------------------

static GROUP_MESSAGE: &[Field] = &[
    Field {
        slug: "carbon-copy-mail",
        title: "Carbon Copy Email (CC)",
        ftype: "text",
        single: Some("tourmaster-tour-cc-mail"),
        description: "Fill the email here to send a copy of an Admin Email for transaction related to this tour.",
        ..F
    },
    Field {
        slug: "payment-notification-title",
        title: "Payment Notification",
        ftype: "title",
        ..F
    },
    Field {
        slug: "enable-payment-notification",
        title: "Enable Payment Notification",
        ftype: "checkbox",
        default: "disable",
        single: Some("tourmaster-payment-notification"),
        description: "By, enabling this option, the system will automatically send a payment notification to customer's email.",
        ..F
    },
    Field {
        slug: "payment-notification-days-before-travel",
        title: "Days Before Travel (Haven't Paid)",
        ftype: "text",
        condition: Some(r#"{"enable-payment-notification":"enable"}"#),
        description: "Send reminder message XX days before the travel date. This will remind customers if customers haven't paid for anything yet.",
        ..F
    },
    Field {
        slug: "deposit-payment-notification-days-before-travel",
        title: "Days Before Travel (Deposit Paid)",
        ftype: "text",
        condition: Some(r#"{"enable-payment-notification":"enable"}"#),
        description: "Send reminder message XX days before the travel date. This will remind customers if customers have paid the deposit but haven' paid the rest amount yet. It will remind customers to pay the rest. If you allow to pay at arrival, you may skip this feature so ones who paid the deposit won't get the reminder message.",
        ..F
    },
    Field {
        slug: "payment-notification-mail-subject",
        title: "Email Subject",
        ftype: "text",
        condition: Some(r#"{"enable-payment-notification":"enable"}"#),
        ..F
    },
    Field {
        slug: "payment-notification-mail-message",
        title: "Email Message",
        ftype: "textarea",
        condition: Some(r#"{"enable-payment-notification":"enable"}"#),
        ..F
    },
    Field {
        slug: "enable-payment-notification-message-admin-copy",
        title: "Send a copy to admin",
        ftype: "checkbox",
        default: "disable",
        condition: Some(r#"{"enable-payment-notification":"enable"}"#),
        ..F
    },
    Field {
        slug: "reminder-message-title",
        title: "Reminder Message",
        ftype: "title",
        ..F
    },
    Field {
        slug: "enable-reminder-message",
        title: "Enable Reminder Message",
        ftype: "checkbox",
        default: "disable",
        single: Some("tourmaster-reminder-message"),
        description: "By, enabling this option, the system will automatically send a reminder message to customer's email.",
        ..F
    },
    Field {
        slug: "reminder-message-days-before-travel",
        title: "Reminder Message Days Before Travel",
        ftype: "text",
        condition: Some(r#"{"enable-reminder-message":"enable"}"#),
        description: "Only number is allowed here.",
        ..F
    },
    Field {
        slug: "reminder-message-mail-subject",
        title: "Email Subject",
        ftype: "text",
        condition: Some(r#"{"enable-reminder-message":"enable"}"#),
        ..F
    },
    Field {
        slug: "reminder-message-mail-message",
        title: "Email Message",
        ftype: "textarea",
        condition: Some(r#"{"enable-reminder-message":"enable"}"#),
        ..F
    },
    Field {
        slug: "enable-reminder-message-admin-copy",
        title: "Send a copy to admin",
        ftype: "checkbox",
        default: "disable",
        condition: Some(r#"{"enable-reminder-message":"enable"}"#),
        ..F
    },
    Field {
        slug: "group-message-title",
        title: "Group Message",
        ftype: "title",
        wrapper_class: "tourmaster-top-margin-wrapper",
        ..F
    },
    Field {
        slug: "group-message-date",
        title: "Group Message Date",
        ftype: "datepicker",
        description: "* To specify the exact group of customer that you want to send the message to.",
        ..F
    },
    Field {
        slug: "group-message-mail-subject",
        title: "Email Subject",
        ftype: "text",
        ..F
    },
    Field {
        slug: "group-message-mail-message",
        title: "Email Message",
        ftype: "textarea",
        ..F
    },
    Field {
        slug: "enable-group-message-admin-copy",
        title: "Send a copy to admin",
        ftype: "checkbox",
        default: "disable",
        description: "Enable this to send the copy of the mail which cusmoter receieve to admin e-mail.",
        ..F
    },
    Field {
        slug: "group-message-submit",
        ftype: "button",
        button_title: "Send Email",
        data_type: "ajax",
        data_action: "tourmaster_submit_group_message",
        data_fields: &[
            "group-message-date",
            "group-message-mail-subject",
            "group-message-mail-message",
            "enable-group-message-admin-copy",
            "group-message-tour-id",
        ],
        ..F
    },
];

// ---------------------------------------------------------------------------
// Tabs: manage-review / add-a-review (second page-option block in PHP)
// ---------------------------------------------------------------------------

static MANAGE_REVIEW: &[Field] = &[Field {
    slug: "manage-review",
    ftype: "manage-review",
    ..F
}];

static ADD_A_REVIEW: &[Field] = &[Field {
    slug: "add-review",
    ftype: "add-review",
    ..F
}];

// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub static TABS: &[Tab] = &[
    Tab {
        slug: "general",
        title: "General",
        fields: GENERAL,
    },
    Tab {
        slug: "tour-settings",
        title: "Tour Settings",
        fields: TOUR_SETTINGS,
    },
    Tab {
        slug: "date-price",
        title: "Date & Price",
        fields: DATE_PRICE,
    },
    Tab {
        slug: "urgency-message",
        title: "Urgency Message",
        fields: URGENCY_MESSAGE,
    },
    Tab {
        slug: "group-message",
        title: "Reminder & Message",
        fields: GROUP_MESSAGE,
    },
    Tab {
        slug: "manage-review",
        title: "Manage Review",
        fields: MANAGE_REVIEW,
    },
    Tab {
        slug: "add-a-review",
        title: "Add A Review",
        fields: ADD_A_REVIEW,
    },
];
