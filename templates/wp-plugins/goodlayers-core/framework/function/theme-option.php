<?php
	/*	
	*	Goodlayers Theme Option
	*	---------------------------------------------------------------------
	*	This file create the class that help you create the controls of the 
	*	theme option
	*	---------------------------------------------------------------------
	*/	

	if( !class_exists('gdlr_core_admin_option') ){
		
		class gdlr_core_admin_option{
			
			// function for elements registration
			private $theme_options = array();
			
			function add_element($options, $order = 10){
				while( !empty($this->theme_options[$order]) ){
					$order++;
				}
				$this->theme_options[$order] = $options;
				ksort($this->theme_options);
			}			
			
			function get_elements(){
				return $this->theme_options;
			}

			private $settings;	
				
			function __construct($settings = array()){

				$this->settings = wp_parse_args($settings, array(
					'main-menu' => true,
					'main-menu-title' => esc_html__('Goodlayers', 'goodlayers-core'),
					'parent-slug' => '',
					'page-title' => esc_html__('Goodlayers Option', 'goodlayers-core'),
					'menu-title' => esc_html__('Goodlayers Option', 'goodlayers-core'),
					'capability' => 'edit_theme_options',
					'slug' => 'goodlayers_main_menu', 
					'icon-url' => GDLR_CORE_URL . '/framework/images/admin-option-icon.png',
					'position' => '2.2',
					'filewrite' => '',
					'container-width' => ''
				));

				// add action to create dashboard
				if( class_exists('gdlr_core_admin_menu') ){
					
					gdlr_core_admin_menu::register_menu(array(
						'main-menu' => $this->settings['main-menu'],
						'main-menu-title' => $this->settings['main-menu-title'], 
						'parent-slug' => $this->settings['parent-slug'],
						'page-title' => $this->settings['page-title'], 
						'menu-title' =>$this->settings['menu-title'], 
						'capability' => $this->settings['capability'], 
						'menu-slug' => $this->settings['slug'], 
						'function' => array(&$this, 'create_theme_option'),
						'icon-url' => $this->settings['icon-url'],
						'position' => $this->settings['position']
					));
					
				}

				add_action('init', array(&$this, 'custom_plugin_option'));

				// add ajax action for the theme option script
				add_action('wp_ajax_save_theme_option', array(&$this, 'save_theme_option_ajax'));
				add_action('wp_ajax_get_theme_option_tab', array(&$this, 'get_theme_option_tab'));
				add_action('wp_ajax_get_theme_option_search', array(&$this, 'get_theme_option_search'));

				add_action('wp_ajax_gdlr_core_theme_option_export', array(&$this, 'theme_option_export'));

				// add the script when opening the theme option page
				add_action('admin_enqueue_scripts', array(&$this, 'load_theme_option_script'));

				// add action to force save style-custom
				add_action('gdlr_core_theme_option_filewrite', array(&$this, 'after_save_theme_option'));
			
			}

			function custom_plugin_option(){

				// add plugins tab
				$options = apply_filters('gdlr_core_admin_option_object', array(), $this->settings['slug']);
				if( !empty($options) ){
					$this->add_element(array(
						'title' => esc_html__('Plugins', 'goodlayers-core'),
						'slug' => 'gdlr_core_admin_option_plugins',
						'icon' => GDLR_CORE_URL . '/framework/images/admin-option-icon.png',
						'options' => $options
					));
				}

			}
			
			// function that enqueue theme option script
			function load_theme_option_script( $hook ){
				if( strpos($hook, 'page_' . $this->settings['slug']) !== false ){
					
					gdlr_core_include_utility_script();
					
					gdlr_core_html_option::include_script(array());
					
					// include the style
					gdlr_core_include_icon_font('font-awesome');
					wp_enqueue_style('gdlr-core-theme-option', GDLR_CORE_URL . '/framework/css/theme-option.css');
					
					$use_font_icons = apply_filters('gdlr_core_use_font_icons', array('font-awesome', 'elegant-font'));
					foreach($use_font_icons as $use_font_icon){
						gdlr_core_include_icon_font($use_font_icon);
					}


					// include the script
					wp_enqueue_script('gdlr-core-theme-option', GDLR_CORE_URL . '/framework/js/theme-option.js', array('jquery'), false, true);
					wp_localize_script('gdlr-core-theme-option', 'gdlr_core_ajax_message', array(
						'ajaxurl' => GDLR_CORE_AJAX_URL,
						'error_head' => esc_html__('An error occurs', 'goodlayers-core'),
						'error_message' => esc_html__('Please refresh the page to try again. If the problem still persists, please contact administrator for this.', 'goodlayers-core'),
						'nonce' => wp_create_nonce('gdlr-core-theme-option-nonce')
					));					
				}
			}

			function create_theme_option(){
				
				// decide the active theme option tab
				if( isset($_GET['nav_order']) ){
					$nav_active = trim($_GET['nav_order']);
				}
				
				if( empty($nav_active) || empty($this->theme_options[$nav_active]) ){
					reset($this->theme_options);
					$nav_active = key($this->theme_options);
				}

				// if import variable is set
				$this->theme_option_import();
				
				echo '<div class="gdlr-core-theme-option-wrapper" ' . gdlr_core_esc_style(array('width'=> $this->settings['container-width'])) . ' >';
				$this->create_theme_option_head($nav_active);
				
				$this->create_theme_option_body($nav_active);
				echo '</div>'; // gdlr-core-theme-option-wrapper

			}
			
			///////////////////////
			// theme option html
			///////////////////////
			function get_theme_option_breadcrumbs($nav_active = ''){
				$ret = '';
				if( $nav_active === '' ){
					$ret .= '<span class="gdlr-core-theme-option-head-breadcrumbs-nav" >' . esc_html__('Search', 'goodlayers-core') . '</span>';
				}else{
					$first_sub_nav = reset($this->theme_options[$nav_active]['options']);
					$ret .= '<span class="gdlr-core-theme-option-head-breadcrumbs-nav" >' . $this->theme_options[$nav_active]['title'] . '</span>';
					$ret .= '<i class="gdlr-core-theme-option-head-breadcrumbs-sep fa fa-angle-right" ></i>';
					$ret .= '<span class="gdlr-core-theme-option-head-breadcrumbs-subnav" >' . $first_sub_nav['title'] . '</span>';
				}
				return $ret;
			}
			function create_theme_option_head($nav_active){
				echo '<div class="gdlr-core-theme-option-head">';
				
				// head nav area
				echo '<div class="gdlr-core-theme-option-head-nav">';
				
				// logo
				echo '<div class="gdlr-core-theme-option-logo gdlr-core-theme-option-left-column gdlr-core-media-image">';
				echo '<img src="' . esc_url(GDLR_CORE_URL . '/framework/images/theme-option-logo.png') . '" alt="theme-option-logo" />';
				echo '</div>';				
				
				// navigation item
				echo '<div class="gdlr-core-theme-option-nav gdlr-core-theme-option-right-column" id="gdlr-core-theme-option-nav">';
				echo '<div class="gdlr-core-theme-option-nav-slides" id="gdlr-core-theme-option-nav-slides"></div>';
				
				foreach( $this->theme_options as $nav_order => $theme_option ){
					$nav_item_class  = 'theme-option-nav-item-' . $theme_option['slug'];
					$nav_item_class .= ($nav_active == $nav_order)? ' gdlr-core-active': '';
					
					echo '<div class="gdlr-core-theme-option-nav-item ' . esc_attr($nav_item_class) . '" data-nav-order="' . esc_attr($nav_order) . '" >';
					if( !empty($theme_option['icon']) ){
						echo '<div class="gdlr-core-theme-option-nav-item-icon gdlr-core-media-image">';
						echo '<img src="' . esc_url($theme_option['icon']) . '" alt="nav-icon" />';
						echo '</div>';
					}
					if( !empty($theme_option['title']) ){
						echo '<div class="gdlr-core-theme-option-nav-item-title">' . gdlr_core_escape_content($theme_option['title']) . '</div>';
					}
					echo '</div>'; // gdlr-core-theme-option-nav-item
				}
				
				echo '<div class="clear"></div>';
				echo '</div>'; // gdlr-core-theme-option-nav
				
				// save button
				echo '<div class="gdlr-core-theme-option-save-button" id="gdlr-core-theme-option-save-button" >' . esc_html__('Save Options', 'goodlayers-core') . '</div>';
				echo '<div class="clear"></div>';
				echo '</div>'; // gdlr-core-theme-option-head-nav
				
				// header sub area
				echo '<div class="gdlr-core-theme-option-head-sub">';
				
				// bread crumbs
				echo '<div class="gdlr-core-theme-option-head-breadcrumbs" id="gdlr-core-theme-option-head-breadcrumbs" >';
				echo gdlr_core_escape_content($this->get_theme_option_breadcrumbs($nav_active));
				echo '</div>';
				
				// search section
				echo '<div class="gdlr-core-theme-option-head-search" >';
				echo '<input type="text" class="gdlr-core-theme-option-head-search-text" id="gdlr-core-theme-option-head-search-text" placeholder="' . esc_html__('Search Options', 'goodlayers-core') . '" />';
				echo '<input type="button" class="gdlr-core-theme-option-head-search-button" id="gdlr-core-theme-option-head-search-button" data-blank-keyword="' . esc_html__('Please fill keywords to search', 'goodlayers-core') . '" />';
				echo '</div>'; // gdlr-core-theme-option-head-search
				
				echo '<div class="clear"></div>';
				echo '</div>'; // gdlr-core-theme-option-head-sub
				
				echo '</div>'; // gdlr-core-theme-option-head	
				
			}
			
			// for creating the theme option body section
			function get_theme_option_subnav($nav_active, $subnav_active = ''){
				$ret = ''; $count = 0;
				if( empty($subnav_active) ){
					reset($this->theme_options[$nav_active]['options']);
					$subnav_active = key($this->theme_options[$nav_active]['options']);
				}
			
				foreach( $this->theme_options[$nav_active]['options'] as $slug => $subnav ){ $count++;
					$subnav_item_class  = 'gdlr-core-theme-option-subnav-item';
					$subnav_item_class .= ($slug == $subnav_active)? ' gdlr-core-active':'';
					$subnav_item_class .= empty($subnav['reload-after'])? '':' gdlr-core-reload-after';
					
					$ret .= '<div class="' . esc_attr($subnav_item_class) . '" data-subnav-slug="' . esc_attr($slug) . '" >' . gdlr_core_escape_content($subnav['title']) . '</div>';
				}		
				return $ret;
			}			
			function get_theme_option_section_content($nav_active, $subnav_active = ''){
				$theme_option_val = get_option($this->theme_options[$nav_active]['slug'], array());
				
				$ret = ''; $count = 0;
				
				if( empty($subnav_active) ){
					reset($this->theme_options[$nav_active]['options']);
					$subnav_active = key($this->theme_options[$nav_active]['options']);
				}
				
				foreach( $this->theme_options[$nav_active]['options'] as $slug => $subnav ){ $count++;
					$ret .= '<div class="gdlr-core-theme-option-section ' . (($count == 1)? 'gdlr-core-active': '') . '" data-section-slug="' . esc_attr($slug) . '" >';
					foreach( $subnav['options'] as $option_slug => $option ){
						$option['slug'] = $option_slug;
						if( isset($theme_option_val[$option_slug]) ){
							$option['value'] = $theme_option_val[$option_slug];
						}
							
						$ret .= gdlr_core_html_option::get_element($option);
					}
					$ret .= '</div>'; // gdlr-core-theme-option-section
				}
				
				$ret .= '<div class="gdlr-core-theme-option-body-content-save" >';
				$ret .= '<div class="gdlr-core-theme-option-save-button" >' . esc_html__('Save Options', 'goodlayers-core') . '</div>';
				$ret .= '</div>';
				
				return $ret;
			}
			function create_theme_option_body($nav_active){
				
				echo '<div class="gdlr-core-theme-option-body">';
				
				// body nav
				echo '<div class="gdlr-core-theme-option-subnav gdlr-core-theme-option-left-column" id="gdlr-core-theme-option-subnav" >';
				echo gdlr_core_escape_content($this->get_theme_option_subnav($nav_active));
				echo '</div>'; // gdlr-core-theme-option-subnav
					
				// body content
				echo '<div class="gdlr-core-theme-option-body-content gdlr-core-theme-option-right-column" id="gdlr-core-theme-option-body-content" >';
				echo gdlr_core_escape_content($this->get_theme_option_section_content($nav_active));		
				echo '</div>'; // gdlr-core-theme-option-body-nav
				
				echo '<div class="clear"></div>';
				echo '</div>'; // gdlr-core-theme-option-body
				
			}
			
			///////////////////////
			// save action
			///////////////////////
			
			// save the option
			function save_theme_option(){

				$theme_options_val = array();
				foreach( $_POST['option'] as $option_key => $option_value ){

					if( ($nav_order = $this->get_option_nav_order($option_key, $option_value)) !== false ){
						$option_slug = $this->theme_options[$nav_order]['slug'];
						if( empty($theme_options_val[$option_slug]) ){
							$theme_options_val[$option_slug] = get_option($option_slug, array());
						}

						// assign values
						$theme_options_val[$option_slug][$option_key] = gdlr_core_process_post_data($option_value);
					}
				}
				
				// save action
				foreach($theme_options_val as $option_slug => $option_value){
					update_option($option_slug, $option_value);
				}
				
				if( $this->settings['filewrite'] ){
					return $this->after_save_theme_option();
				}
			}			
			
			// write data
			function after_save_theme_option(){
				do_action('gdlr_core_after_save_theme_option');

				if( empty($this->settings['filewrite']) ){
					return true;
				}
				
				global $gdlr_core_font_loader;
				if( empty($gdlr_core_font_loader) ){
					$gdlr_core_font_loader = new gdlr_core_font_loader();
				}
				
				$use_font_list = array();

				$data = apply_filters('gdlr_core_theme_option_top_file_write', '', $this->settings['slug']);

				foreach( $this->theme_options as $nav =>$theme_option ){ // main nav
					$theme_option_val = get_option($theme_option['slug'], array());
					foreach( $theme_option['options'] as $options ){ // sub nav
						foreach( $options['options'] as $option_slug => $option ){ // content	

							if( !empty($option['data-type']) && $option['data-type'] == 'font' && !empty($theme_option_val[$option_slug]) ){ $use_font_list[$theme_option_val[$option_slug]] = ''; }

							if( empty($option['selector']) ) continue; 

							if( !empty($theme_option_val[$option_slug]) || (isset($theme_option_val[$option_slug]) && $theme_option_val[$option_slug] === '0') ){

								// format the variable
								if( !empty($option['data-type']) && $option['data-type'] == 'font' ){
									$theme_option_val[$option_slug] = $gdlr_core_font_loader->get_font_family_css($theme_option_val[$option_slug]);
								}

								$data .= gdlr_core_option_to_css($option_slug, $option, $theme_option_val);

							}
						}
					}
				}
				
				// for skin saving
				$data .= gdlr_core_skin_settings::get_skins_style();
				
				// for font saving
				$gdlr_core_font_loader->update_used_font($use_font_list);
				$data .= $gdlr_core_font_loader->get_custom_fontface() . "\n";

				// for custom value
				$data .= apply_filters('gdlr_core_theme_option_bottom_file_write', '', $this->settings['slug']);
				
				$fs = new gdlr_core_file_system();
				return $fs->write($this->settings['filewrite'], $data);
			}
			function set_customize_preview_font(){

				global $gdlr_core_font_loader;
				if( empty($gdlr_core_font_loader) ){
					$gdlr_core_font_loader = new gdlr_core_font_loader();
				}

				$use_font_list = array();

				foreach( $this->theme_options as $nav =>$theme_option ){ // main nav
					$theme_option_val = get_option($theme_option['slug'], array());
					foreach( $theme_option['options'] as $options ){ // sub nav
						foreach( $options['options'] as $option_slug => $option ){ // content	
							if( !empty($theme_option_val[$option_slug]) && !empty($option['data-type']) 
								&& $option['data-type'] == 'font' ){
								
								$use_font_list[$theme_option_val[$option_slug]] = '';
							}
						}
					}
				}

				// for font saving
				$gdlr_core_font_loader->set_used_font($use_font_list);

			}
			function get_customize_preview_css(){

				global $gdlr_core_font_loader;
				if( empty($gdlr_core_font_loader) ){
					$gdlr_core_font_loader = new gdlr_core_font_loader();
				}

				$data = apply_filters('gdlr_core_theme_option_top_file_write', '', $this->settings['slug']);

				foreach( $this->theme_options as $nav =>$theme_option ){ // main nav
					$theme_option_val = get_option($theme_option['slug'], array());
					foreach( $theme_option['options'] as $options ){ // sub nav
						foreach( $options['options'] as $option_slug => $option ){ // content	

							if( empty($option['selector']) ) continue; 

							if( !empty($theme_option_val[$option_slug]) || (isset($theme_option_val[$option_slug]) && $theme_option_val[$option_slug] === '0') ){

								// format the variable
								if( !empty($option['data-type']) && $option['data-type'] == 'font' ){
									$theme_option_val[$option_slug] = $gdlr_core_font_loader->get_font_family_css($theme_option_val[$option_slug]);
								}

								$data .= gdlr_core_option_to_css($option_slug, $option, $theme_option_val);
							}
						}
					}
				}
				
				// for skin 
				$data .= gdlr_core_skin_settings::get_skins_style();

				// for fontface
				$data .= $gdlr_core_font_loader->get_custom_fontface() . "\n";

				// for custom value
				$data .= apply_filters('gdlr_core_theme_option_bottom_file_write', '', $this->settings['slug']);

				return $data;
			}
			
			///////////////////////
			// ajax call
			///////////////////////
			function get_option_nav_order($option_key, $option_value){
				foreach( $this->theme_options as $nav_order => $nav_options ){
					foreach( $nav_options['options'] as $key => $options ){
						if( !empty($options['options'][$option_key]) ){
							
							// validate pixel type
							if( !empty($options['options'][$option_key]['data-type']) && $options['options'][$option_key]['data-type'] == 'pixel' ){
								preg_match('/^\-?\d+\.?\d*(px|em|pt|\%)$/', $option_value, $matches);
								if( empty($matches) ){
									return false;
								}
							}

							return $nav_order;
						}
					}
				}
				return false;
			}
			function save_theme_option_ajax(){

				if( !check_ajax_referer('gdlr-core-theme-option-nonce', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}			
				
				if( empty($_POST['option']) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('An Error Occurs', 'goodlayers-core'),
						'message' => esc_html__('No variable for saving process, please refresh the page to try again.', 'goodlayers-core')
					)));
				}else{
					
					$status = $this->save_theme_option();
					
					if( $status === true ){
						die(json_encode(array(
							'status' => 'success',
							'head' => esc_html__('Options Saved!', 'goodlayers-core')
						)));
					}else{
						die(json_encode($status));
					}
				}
			}			
			
			function get_theme_option_tab(){

				if( !check_ajax_referer('gdlr-core-theme-option-nonce', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}			
				
				$nav_order = empty($_POST['nav_order'])? '': trim($_POST['nav_order']);
				if( empty($this->theme_options[$nav_order]) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('An Error Occurs', 'goodlayers-core'),
						'message' => esc_html__('Unable to obtain the tab variable, please refresh the page to try again.', 'goodlayers-core')
					)));
				}else{
					
					if( !empty($_POST['option']) ){
						$this->save_theme_option();
					}
					
					$subnav_active = empty($_POST['subnav_order'])? '': trim($_POST['subnav_order']);
					die(json_encode(array(
						'status' => 'success',
						'breadcrumbs' => $this->get_theme_option_breadcrumbs($nav_order, $subnav_active),
						'subnav' => $this->get_theme_option_subnav($nav_order, $subnav_active),
						'content' => $this->get_theme_option_section_content($nav_order, $subnav_active)
					)));
				}
			}
			
			function get_theme_option_search_content($keyword = ''){
				if( empty($keyword) ) return '';

				$count = 0;
				
				$ret  = '<div class="gdlr-core-theme-option-section gdlr-core-active" >';
				foreach( $this->theme_options as $nav =>$theme_option ){ // main nav
					$theme_option_val = get_option($theme_option['slug'], array());
					foreach( $theme_option['options'] as $options ){ // sub nav
						foreach( $options['options'] as $option_slug => $option ){ // content
							if( stripos($option_slug, $keyword) !== false || stripos($option['title'], $keyword) !== false  ){
								$count++;
								
								$option['slug'] = $option_slug;
								if( isset($theme_option_val[$option_slug]) ){
									$option['value'] = $theme_option_val[$option_slug];
								}

								$ret .= gdlr_core_html_option::get_element($option);
							}
						}
					}
				}
				
				if( $count == 0 ){
					$ret .= '<div class="gdlr-core-theme-option-search-not-found">';
					$ret .= '<div class="gdlr-core-head">' . esc_html__('No results match the keyword', 'goodlayers-core') . ' "' . esc_html($keyword) . '"</div>';
					$ret .= '<div class="gdlr-core-tail">' . esc_html__('Please try again.', 'goodlayers-core') . '</div>';
					$ret .= '</div>';
				}
				$ret .= '</div>'; // gdlr-core-theme-option-section
				
				if( $count > 0 ){
					$ret .= '<div class="gdlr-core-theme-option-body-content-save" >';
					$ret .= '<div class="gdlr-core-theme-option-save-button" >' . esc_html__('Save Options', 'goodlayers-core') . '</div>';
					$ret .= '</div>';
				}
				
				return $ret;				
			}
			function get_theme_option_search(){

				if( !check_ajax_referer('gdlr-core-theme-option-nonce', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}			

				if( empty($_POST['keyword']) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('An Error Occurs', 'goodlayers-core'),
						'message' => esc_html__('Unable to obtain the tab variable, please refresh the page to try again.', 'goodlayers-core')
					)));
				}else{
					
					if( !empty($_POST['option']) ){
						$this->save_theme_option();
					}
					
					die(json_encode(array(
						'status' => 'success',
						'breadcrumbs' => $this->get_theme_option_breadcrumbs(),
						'subnav' => '',
						'content' => $this->get_theme_option_search_content(trim($_POST['keyword']))
					)));
				}
			}

			//////////////////
			// import export
			//////////////////

			function theme_option_import(){
				
				if( !empty($_FILES['gdlr-core-import']['tmp_name']) ){

					$fs = new gdlr_core_file_system();
					$import_options = $fs->read($_FILES['gdlr-core-import']['tmp_name']);
					$import_options = json_decode($import_options, true);
					if( is_array($import_options) ){
						foreach( $import_options as $option_slug => $option ){
							if( 'gdlr-core-pb-custom-template' == $option_slug ){
								gdlr_core_page_builder_custom_template::add_template($option);
							}else{
								update_option($option_slug, $option);
							}
						}
							
						if( $this->settings['filewrite'] ){
							$this->after_save_theme_option();
						}
					}

				}

			}
			function theme_option_export(){
				if( !check_ajax_referer('gdlr_core_html_option', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}

				
				if( empty($_POST['options']) || $_POST['options'] == 'all' ){
					$content = array();
					foreach( $this->theme_options as $theme_option ){
						$content[$theme_option['slug']] = get_option($theme_option['slug'], array());
					}
					$extras = array();
					// $extras = array('tourmaster_general', 'tourmaster_color', 'tourmaster_plugin', 'tourmaster_room_general', 'tourmaster_room_color');
					
					foreach( $extras as $slug ){
						$content[$slug] = get_option($slug, array());
					}

					$filename = 'theme-options.json';
				}else{
					if( $_POST['options'] == 'widget' ){
						global $wpdb;
						$content = array();
						$content['gdlrcst_sidebar_name'] = get_option('gdlrcst_sidebar_name','');
						$content['sidebars_widgets'] = get_option('sidebars_widgets','');

						$widget_options = $wpdb->get_results("SELECT option_name FROM $wpdb->options WHERE option_name LIKE 'widget_%'");
						foreach( $widget_options as $widget_option ){
							$content[$widget_option->option_name] = get_option($widget_option->option_name, '');
						}
					}else if( $_POST['options'] == 'page-builder-template' ){
						$template_slug = 'gdlr-core-pb-custom-template';
						$content = array(
							$template_slug => get_option($template_slug, array())
						);
					}else{
						$content = array(
							$_POST['options'] => get_option($_POST['options'], array())
						);

					}
					$filename = $_POST['options'] . '.json';
				}

				$fs = new gdlr_core_file_system();
				$fs_status = $fs->write(GDLR_CORE_LOCAL . '/include/js/theme-option.json', json_encode($content));

				if( $fs_status === true ){
					die(json_encode(array(
						'status' => 'success',
						'url' => GDLR_CORE_URL . '/include/js/theme-option.json',
						'filename' => $filename
					)));
				}else{
					die(json_encode(array(
						'status' => 'success-2',
						'content' => json_encode($content),
						'filename' => $filename
						
					)));
				}
			}

		} // gdlr_core_admin_option
		
	} // class_exists

