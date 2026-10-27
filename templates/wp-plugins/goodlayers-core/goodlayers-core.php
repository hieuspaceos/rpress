<?php
	/*
	Plugin Name: Goodlayers Core
	Plugin URI: 
	Description: A core plugin for Goodlayers' Theme ( 3rd generation )
	Version: 2.1.8
	Author: Goodlayers
	Author URI: http://www.goodlayers.com
	License: 
	*/

	// define necessary variable for the site.
	define('GDLR_CORE_URL', plugins_url('', __FILE__));
	define('GDLR_CORE_LOCAL', dirname(__FILE__));
	
	$gdlr_core_item_pdb = '30px';
	
	///////////////////////////////
	// include core system file
	///////////////////////////////
	include_once(GDLR_CORE_LOCAL . '/framework/function/file-system.php');
	include_once(GDLR_CORE_LOCAL . '/framework/function/media.php');
	include_once(GDLR_CORE_LOCAL . '/framework/function/utility.php');
	include_once(GDLR_CORE_LOCAL . '/framework/function/sidebar-generator.php');
	include_once(GDLR_CORE_LOCAL . '/framework/function/skin-settings.php');

	// load only in admin pages
	if( is_admin() ){
		include_once(GDLR_CORE_LOCAL . '/framework/function/revision.php'); 

		include_once(GDLR_CORE_LOCAL . '/framework/function/admin-menu.php');
		gdlr_core_admin_menu::init();

		include_once(GDLR_CORE_LOCAL . '/framework/function/getting-start.php');
		include_once(GDLR_CORE_LOCAL . '/framework/importer/parsers.php');
		include_once(GDLR_CORE_LOCAL . '/framework/importer/importer.php');
		
		include_once(GDLR_CORE_LOCAL . '/framework/function/user-option.php');
		include_once(GDLR_CORE_LOCAL . '/framework/function/page-option.php');
		
		include_once(GDLR_CORE_LOCAL . '/framework/function/html-option.php');
		include_once(GDLR_CORE_LOCAL . '/framework/function/tax-option.php');
		add_filter('wp_ajax_gdlr_get_gallery_options', 'gdlr_core_html_option::get_gallery_options');

		include_once(GDLR_CORE_LOCAL . '/template/prebuilt-block.php');

		// plugin script/style
		include_once(GDLR_CORE_LOCAL . '/plugins/combine.php');
		include_once(GDLR_CORE_LOCAL . '/plugins/rankmath/rankmath.php');
	}
	
	// for page builder
	include_once(GDLR_CORE_LOCAL . '/framework/function/page-builder.php'); 
	include_once(GDLR_CORE_LOCAL . '/framework/function/page-builder-options.php'); 
	include_once(GDLR_CORE_LOCAL . '/framework/function/page-builder-wrapper.php'); 
	include_once(GDLR_CORE_LOCAL . '/framework/function/page-builder-element.php');
	include_once(GDLR_CORE_LOCAL . '/framework/function/page-builder-template.php');
	include_once(GDLR_CORE_LOCAL . '/framework/function/page-builder-custom-template.php');
	include_once(GDLR_CORE_LOCAL . '/framework/function/page-builder-sync-template.php');
	include_once(GDLR_CORE_LOCAL . '/framework/function/framework2-compatibility.php');
	// include_once(GDLR_CORE_LOCAL . '/framework/function/template-settings.php');
	
	include_once(GDLR_CORE_LOCAL . '/include/header.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/privacy.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb-wrapper/pb-wrapper-background.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb-wrapper/pb-wrapper-sidebar.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb-wrapper/pb-wrapper-column.php'); 
	// include_once(GDLR_CORE_LOCAL . '/include/pb-wrapper/pb-wrapper-flexbox.php'); 

	include_once(GDLR_CORE_LOCAL . '/include/pb/blog-item.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/blog-style.php');

	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-accordion.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-audio.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-blockquote.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-button.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-alert-box.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-blog.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-call-to-action.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-chart.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-code.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-column-service.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-columnize.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-countdown.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-counter.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-custom-menu.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-divider.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-event.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-feature-box.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-feature-content.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-flipbox.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-gallery.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-featured-gallery.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-hover-box.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-hover-content.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-icon.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-icon-list.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-image.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-image-content.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-instagram.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-opening-hours.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-price-plan-cf7.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-page-list.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-port-info.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-post-slider.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-price-list.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-price-table.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-promo-box.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-road-map.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-shape-divider.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-skill-bar.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-skill-circle.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-social-network.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-social-share.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-space.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-stunning-text.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-tab.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-tab-feature.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-tab-feature2.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-tab-feature3.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-tab-feature-vertical.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-text-box.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-text-script.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-testimonial.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-title.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-timeline.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-toggle-box.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-type-animation.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-video.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-widget.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-wordpress-editor-content.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/pb-element-food-menu.php');

	include_once(GDLR_CORE_LOCAL . '/include/pb/post-type/post-title.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb/post-type/post-info.php');

	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/pb-element-depicter.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/product-settings.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/product-item.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/product-style.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/pb-element-product.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/product-table-item.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/pb-element-product-table.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/pb-element-product-category.php'); 

	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/pb-element-breadcrumbs.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/pb-element-contact-form-7.php');
	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/pb-element-layer-slider.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/pb-element-newsletter.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/pb-element-master-slider.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/pb-element-revolution-slider.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/pb-plugins/pb-element-wp-google-map-plugin.php'); 

	include_once(GDLR_CORE_LOCAL . '/include/element/dropcap.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/element/dropdown-tab.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/element/widget-shortcode.php'); 

	/* include shortcode bar */
	include_once(GDLR_CORE_LOCAL . '/framework/function/shortcode-list.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/element/shortcode-list.php'); 

	/* include widget */
	include_once(GDLR_CORE_LOCAL . '/framework/function/widget-util.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/widget/category-background-widget.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/widget/custom-menu-widget.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/widget/recent-post-widget.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/widget/recent-comment-widget.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/widget/recent-event-widget.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/widget/post-slider-widget.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/widget/opening-hour-widget.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/widget/video-widget.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/widget/instagram-widget.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/widget/newsletter-widget.php'); 
	include_once(GDLR_CORE_LOCAL . '/include/widget/plain-text-widget.php'); 

	include_once(GDLR_CORE_LOCAL . '/plugins/icon-font-controls.php');

	// init the font loader object
	include_once(GDLR_CORE_LOCAL . '/framework/function/font-loader.php');
	if( !is_admin() ){ 
		add_action('wp_enqueue_scripts', 'gdlr_core_enqueue_google_font'); 
		if( !function_exists('gdlr_core_enqueue_google_font') ){
			function gdlr_core_enqueue_google_font() {
				global $gdlr_core_font_loader;
				if( empty($gdlr_core_font_loader) ){
					$gdlr_core_font_loader = new gdlr_core_font_loader();
				}
				$gdlr_core_font_loader->google_font_enqueue();
			}
		}
	}
	
	// for customizer
	include_once(GDLR_CORE_LOCAL . '/framework/function/theme-option.php');
	include_once(GDLR_CORE_LOCAL . '/framework/function/customizer.php');
	
	// menu edit class
	include_once(GDLR_CORE_LOCAL . '/framework/function/navigation-menu.php');
	
	// init the ajax variable for wpml compatibility
	add_action('plugins_loaded', 'init_goodlayers_core_system', 1);
	if( !function_exists('init_goodlayers_core_system') ){
		function init_goodlayers_core_system(){
			global $sitepress;
			if( !empty($sitepress) ){
				define('GDLR_CORE_AJAX_URL', admin_url('admin-ajax.php?lang=' . $sitepress->get_current_language()));
			}else{
				define('GDLR_CORE_AJAX_URL', admin_url('admin-ajax.php'));
			}
		}
	}
	
	// create the page builder
	add_action('after_setup_theme', 'gdlr_init_goodlayers_core_elements');
	if( !function_exists('gdlr_init_goodlayers_core_elements') ){
		function gdlr_init_goodlayers_core_elements(){
			if( is_admin() ){
				$revision_num = 5;
				new gdlr_core_revision($revision_num);
			}
			new gdlr_core_page_builder();

			remove_action('wp_enqueue_scripts', 'gdlr_core_init_google_fonts');
		}
	}

	// enqueue necessay style for front end
	if( !is_admin() ){ 
		add_action('wp_enqueue_scripts', 'gdlr_core_front_script'); 
	}
	if( !function_exists('gdlr_core_front_script') ){
		function gdlr_core_front_script( $admin = false ){

			// enqueue icon font
			$use_font_icons = apply_filters('gdlr_core_use_font_icons', array('font-awesome', 'elegant-font'));
			foreach($use_font_icons as $use_font_icon){
				gdlr_core_include_icon_font($use_font_icon);
			}

			// enqueue selected script
			$filemtime = filemtime(GDLR_CORE_LOCAL . '/plugins/style.css');
			wp_enqueue_style('gdlr-core-plugin', GDLR_CORE_URL . '/plugins/style.css', array(), $filemtime);

			$filemtime = filemtime(GDLR_CORE_LOCAL . '/plugins/script.js');
			wp_enqueue_script('gdlr-core-plugin', GDLR_CORE_URL . '/plugins/script.js', array('jquery'), $filemtime, true);

			// enqueue main goodlayers script
			$gdlr_core_pbf = array( 
				'admin' => $admin,
				'video' => array('width' => '640', 'height' => '360'),
				'ajax_url' => GDLR_CORE_AJAX_URL
			);

			$lightbox = apply_filters('gdlr_core_lightbox_type', 'ilightbox-dark');
			if( strpos($lightbox, 'ilightbox-') !== false ){
				$gdlr_core_pbf['ilightbox_skin'] = str_replace('ilightbox-', '', $lightbox);
			}
			wp_enqueue_style('gdlr-core-page-builder', GDLR_CORE_URL . '/include/css/page-builder.css');
			wp_enqueue_script('gdlr-core-page-builder', GDLR_CORE_URL . '/include/js/page-builder.js', array('jquery'), '1.3.9', true);
			wp_localize_script('gdlr-core-page-builder', 'gdlr_core_pbf', $gdlr_core_pbf);

			// enqueue plugin script for page builder admin page
			if( $admin ){
				do_action('gdlr_core_front_script');
			}

		}
	}

	if( !function_exists('gdlr_core_generate_combine_script') ){
		function gdlr_core_generate_combine_script( $script_included ){
			
			$script_included = wp_parse_args($script_included, array(
				'include' => 'flexslider',
				'lightbox' => 'ilightbox-dark'
			));

			$fs = new gdlr_core_file_system();

			$style = gdlr_core_get_combine_plugin_style($script_included);
			$fs->write(GDLR_CORE_LOCAL . '/plugins/style.css', $style);

			$script = gdlr_core_get_combine_plugin_script($script_included);
			$fs->write(GDLR_CORE_LOCAL . '/plugins/script.js', $script);

		}
	}	

	// add body class for page builder
	add_filter('body_class', 'gdlr_core_body_class');
	if( !function_exists('gdlr_core_body_class') ){
		function gdlr_core_body_class( $classes ) {
			$classes[] = 'gdlr-core-body';
			return $classes;
		}
	}

	add_action('init', 'gdlr_core_load_textdomain');
	if( !function_exists('gdlr_core_load_textdomain') ){
		function gdlr_core_load_textdomain() {
		  load_plugin_textdomain('goodlayers-core', false, plugin_basename(dirname(__FILE__)) . '/languages'); 
		}
	}

	// create post category thumbnail
	add_action('init', 'gdlr_core_set_custom_tax_option');
	if( !function_exists('gdlr_core_set_custom_tax_option') ){
		function gdlr_core_set_custom_tax_option(){
			if( is_admin() && class_exists('gdlr_core_taxonomy_option') ){
				new gdlr_core_taxonomy_option(array(
					'taxonomy' => 'category',
					'slug' => 'gdlr-core-meta',
					'options' => array(
						'thumbnail' => array(
							'title' => esc_html__('Upload Thumbnail', 'goodlayers-core'),
							'type' => 'upload'
						)
					)
				));
			}
		}
	} // function_exists
	