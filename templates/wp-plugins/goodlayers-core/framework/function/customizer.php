<?php
	/*	
	*	Goodlayers Customizer
	*	---------------------------------------------------------------------
	*	File which creates the customizer page
	*	---------------------------------------------------------------------
	*/

	// sanitizing 
	// https://codex.wordpress.org/Function_Reference/sanitize_text_field
	
	if( !class_exists('gdlr_core_theme_customizer') ){
		
		class gdlr_core_theme_customizer{
			
			private $gdlr_core_admin_option = array();
			private $post_variable = array();
			private $google_font_link = false;

			// initiate the instance
			function __construct($gdlr_core_admin_option){
				
				if( empty($gdlr_core_admin_option) ) return;
				$this->gdlr_core_admin_option = $gdlr_core_admin_option;
				
				add_action('customize_controls_enqueue_scripts', array(&$this, 'enqueue_control_script'));
				add_action('customize_register', array(&$this, 'register_customizer_option'));
				add_action('customize_preview_init', array(&$this, 'enqueue_display_script'));
				if( is_customize_preview() ){
					add_action('wp_head', array(&$this, 'add_style_custom'));
				}

				add_action('customize_save_after', array(&$this, 'after_save_theme_option'));
				
				include_once(GDLR_CORE_LOCAL . '/framework/function/customizer-controls.php');
			}
			
			// enqueue customizer script for customizer page
			function enqueue_control_script(){
				
				wp_enqueue_style('gdlr-core-customizer-controls', GDLR_CORE_URL . '/framework/css/customizer-controls.css');
				
				wp_enqueue_script('gdlr-core-utility', GDLR_CORE_URL . '/framework/js/utility.js', array('jquery', 'jquery-ui-slider'), false, true);
				wp_enqueue_script('gdlr-core-customizer-controls', GDLR_CORE_URL . '/framework/js/customizer-controls.js', array('jquery', 'jquery-ui-slider'), false, true);
				
				// for registering the conditions
				$option_condition = array();
				
				$theme_options = $this->gdlr_core_admin_option->get_elements();
				foreach( $theme_options as $main_section ){
					if( isset($main_section['customizer']) && $main_section['customizer'] === false ) continue;
					
					foreach( $main_section['options'] as $ss_slug => $sub_section ){
						if( isset($sub_section['customizer']) && $sub_section['customizer'] === false ) continue;
							
						foreach( $sub_section['options'] as $option_slug => $option ){
							if( isset($option['customizer']) && $option['customizer'] === false ) continue;
							
							if( !empty($option['condition']) ){
								if( empty($option_condition[$ss_slug]) ){
									$option_condition[$ss_slug] = array();
								}
								$option_condition[$ss_slug][$option_slug] = $option['condition'];
							}
						}
					}
				}
				if( !empty($option_condition) ){
					wp_localize_script('gdlr-core-customizer-controls', 'gdlr_core_customizer_controls', $option_condition);
				}

			}
			function enqueue_display_script(){
				
				$gdlr_core_customizer = array();
				if( !empty($this->post_variable) ){
					$gdlr_core_customizer['post_val'] = $this->post_variable;
				}
				if( !empty($this->google_font_link) ){
					global $gdlr_core_font_loader;
					if( empty($gdlr_core_font_loader) ){
						$gdlr_core_font_loader = new gdlr_core_font_loader();
					}
					$gdlr_core_customizer['google_font_link'] = $gdlr_core_font_loader->get_google_font('link-list');
				}
				
				$customizer_to_file = get_theme_support('gdlr_core_customizer_to_file');

				if( $customizer_to_file ){
					$upload_dir = wp_upload_dir();
					$filename = '/gdlr_core_customizer.js';
					$data = 'var gdlr_core_customizer = ' . json_encode($gdlr_core_customizer);
					$fs = new gdlr_core_file_system();
					$fs->write($upload_dir['basedir'] . $filename, $data);
					if( file_exists($upload_dir['basedir'] . $filename) ){
						$filemtime = filemtime($upload_dir['basedir'] . $filename);
						wp_enqueue_script('gdlr-core-customizer-data', $upload_dir['baseurl'] . $filename . '?' . $filemtime, array('jquery'), false, true);
					}
					wp_enqueue_script('gdlr-core-customizer', GDLR_CORE_URL . '/framework/js/customizer.js', array('jquery'), false, true);
				}else{
					wp_enqueue_script('gdlr-core-customizer', GDLR_CORE_URL . '/framework/js/customizer.js', array('jquery'), false, true);
					wp_localize_script('gdlr-core-customizer', 'gdlr_core_customizer', $gdlr_core_customizer);
				}

				// to change the google font query
				$this->gdlr_core_admin_option->set_customize_preview_font();
			}
			function add_style_custom(){
				echo '<style>' . $this->gdlr_core_admin_option->get_customize_preview_css() . '</style>';
			}

			// register theme option to customizer
			function register_customizer_option($wp_customize){
				
				$theme_options = $this->gdlr_core_admin_option->get_elements();
				$priority = 900;
				foreach( $theme_options as $main_section ){
					if( isset($main_section['customizer']) && $main_section['customizer'] === false ) continue;

					$wp_customize->add_panel($main_section['slug'], array(
						'title' => $main_section['title'],
						'priority' => $priority,
					));	
					$priority++;
					
					foreach( $main_section['options'] as $ss_slug => $sub_section ){
						if( isset($sub_section['customizer']) && $sub_section['customizer'] === false ) continue;
						
						$wp_customize->add_section($ss_slug, array(
							'title' => $sub_section['title'],
							'panel' => $main_section['slug'],
							'capability' => 'edit_theme_options'
						));
						
						foreach( $sub_section['options'] as $option_slug => $option ){
							if( isset($option['customizer']) && $option['customizer'] === false ) continue;
							
							// set the default variable
							$cz_control = ''; 
							$cz_controls = array(
								'label' => $option['title'],
								'section' => $ss_slug,
								'settings' => $main_section['slug'] . "[$option_slug]"
							);
							$cz_settings = array(
								'default' => empty($option['default'])? '': $option['default'],
								'type' => 'option',
								'transport' => 'refresh'
							);
							
							// apply the variable base on input type
							if( $option['type'] == 'colorpicker' ){
								$cz_control = 'WP_Customize_Color_Control';
								$cz_settings['sanitize_callback'] = 'sanitize_hex_color';
							}else if( $option['type'] == 'text' ){
								$cz_control = 'WP_Customize_Control';
								if( !empty($option['data-input-type']) ){
									$cz_controls['input_attrs'] = array(
										'data-input-type' => $option['data-input-type']
									);
								}
							}else if( $option['type'] == 'textarea' ){
								$cz_control = 'WP_Customize_Control';
								$cz_controls['type'] = 'textarea';
							}else if( $option['type'] == 'combobox' ){
								$cz_control = 'WP_Customize_Control';
								$cz_controls['type'] = 'select';
								$cz_controls['choices'] = $option['options'];

								if( $cz_controls['choices'] == 'sidebar' ){
									$cz_controls['choices'] = gdlr_core_sidebar_generator::get_sidebars();
								}else if( $cz_controls['choices'] == 'thumbnail-size' ){
									$cz_controls['choices'] = gdlr_core_get_thumbnail_list();
								}else if( $cz_controls['choices'] == 'post_type' ){
									$cz_controls['choices'] = gdlr_core_get_post_list($option['options-data']);
								}
							}else if( $option['type'] == 'radioimage' || $option['type'] == 'radioimage-frame' ){
								$cz_control = 'GDLR_Core_Customize_RadioImage_Control';
								
								if( $option['options'] == 'text-align' ){
									$cz_controls['choices'] = array(
										'left' => GDLR_CORE_URL . '/include/images/text-align/left.png',
										'center' => GDLR_CORE_URL . '/include/images/text-align/center.png',
										'right' => GDLR_CORE_URL . '/include/images/text-align/right.png'
									);
									$cz_controls['input_attrs'] = array(
										'max-width' => '61px',
										'type' => 'radioimage-frame'
 									);
								}else if( $option['options'] == 'sidebar' ){
									$cz_controls['choices'] = array(
										'none' => GDLR_CORE_URL . '/include/images/sidebar/none.jpg',
										'left' => GDLR_CORE_URL . '/include/images/sidebar/left.jpg',
										'right' => GDLR_CORE_URL . '/include/images/sidebar/right.jpg',
										'both' => GDLR_CORE_URL . '/include/images/sidebar/both.jpg',
									);
								}else if($option['options'] == 'hover-icon-link' ){
									$cz_controls['choices'] = array(
										'arrow_right-up' => GDLR_CORE_URL . '/include/images/hover-icon/link/arrow_right-up.jpg',
										'fa fa-external-link' => GDLR_CORE_URL . '/include/images/hover-icon/link/fa-external-link.jpg',
										'fa fa-external-link-square' => GDLR_CORE_URL . '/include/images/hover-icon/link/fa-external-link-square.jpg',
										'fa fa-link' => GDLR_CORE_URL . '/include/images/hover-icon/link/fa-link.jpg',
										'icon_link' => GDLR_CORE_URL . '/include/images/hover-icon/link/icon_link.jpg',
										'icon_link_alt' => GDLR_CORE_URL . '/include/images/hover-icon/link/icon_link_alt.jpg'
									);
								}else if( $option['options'] == 'hover-icon-image' ){
									$cz_controls['choices'] = array(
										'arrow_expand' => GDLR_CORE_URL . '/include/images/hover-icon/image/arrow_expand.jpg',
										'fa fa-expand' => GDLR_CORE_URL . '/include/images/hover-icon/image/fa-expand.jpg',
										'fa fa-picture-o' => GDLR_CORE_URL . '/include/images/hover-icon/image/fa-picture-o.jpg',
										'fa fa-plus' => GDLR_CORE_URL . '/include/images/hover-icon/image/fa-plus.jpg',
										'fa fa-plus-circle' => GDLR_CORE_URL . '/include/images/hover-icon/image/fa-plus-circle.jpg',
										'fa fa-search' => GDLR_CORE_URL . '/include/images/hover-icon/image/fa-search.jpg',
										'fa fa-search-plus' => GDLR_CORE_URL . '/include/images/hover-icon/image/fa-search-plus.jpg',
										'icon_plus' => GDLR_CORE_URL . '/include/images/hover-icon/image/icon_plus.jpg',
										'icon_plus_alt2' => GDLR_CORE_URL . '/include/images/hover-icon/image/icon_plus_alt2.jpg',
										'icon_search' => GDLR_CORE_URL . '/include/images/hover-icon/image/icon_search.jpg',
										'icon_zoom-in_alt' => GDLR_CORE_URL . '/include/images/hover-icon/image/icon_zoom-in_alt.jpg',
									);
								}else if( $option['options'] == 'hover-icon-video' ){
									$cz_controls['choices'] = array(
										'fa fa-file-video-o' => GDLR_CORE_URL . '/include/images/hover-icon/video/fa-file-video-o.jpg',
										'fa fa-film' => GDLR_CORE_URL . '/include/images/hover-icon/video/fa-film.jpg',
										'fa fa-play' => GDLR_CORE_URL . '/include/images/hover-icon/video/fa-play.jpg',
										'fa fa-play-circle' => GDLR_CORE_URL . '/include/images/hover-icon/video/fa-play-circle.jpg',
										'fa fa-play-circle-o' => GDLR_CORE_URL . '/include/images/hover-icon/video/fa-play-circle-o.jpg',
										'fa fa-video-camera' => GDLR_CORE_URL . '/include/images/hover-icon/video/fa-video-camera.jpg',
										'icon_film' => GDLR_CORE_URL . '/include/images/hover-icon/video/icon_film.jpg',
									);
								}else{

									if( $option['options'] == 'pattern' ){
										$option['options'] = array(
											'pattern-1' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-1.png',
											'pattern-2' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-2.png',
											'pattern-3' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-3.png',
											'pattern-4' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-4.png',
											'pattern-5' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-5.png',
											'pattern-6' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-6.png',
											'pattern-7' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-7.png',
											'pattern-8' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-8.png',
											'pattern-9' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-9.png',
											'pattern-10' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-10.png',
											'pattern-11' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-11.png',
											'pattern-12' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-12.png',
											'pattern-13' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-13.png',
											'pattern-14' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-14.png',
											'pattern-15' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-15.png',
											'pattern-16' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-16.png',
											'pattern-17' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-17.png',
											'pattern-18' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-18.png',
											'pattern-19' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-19.png',
											'pattern-20' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-20.png',
											'pattern-21' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-21.png',
											'pattern-22' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-22.png',
											'pattern-23' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-23.png',
											'pattern-24' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-24.png',
											'pattern-25' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-25.png',
											'pattern-26' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-26.png'
										);
									}
									$cz_controls['choices'] = $option['options'];
									$cz_controls['input_attrs'] = array(
										'max-width' => empty($option['max-width'])? '': $option['max-width'],
										'type' => $option['type']
									);
								}

							}else if( $option['type'] == 'upload' ){
								$cz_control = 'WP_Customize_Image_Control';
								$cz_settings['sanitize_callback'] = array(&$this, 'sanitize_customizer_media');
							}else if( $option['type'] == 'checkbox' ){
								$cz_control = 'GDLR_Core_Customize_Checkbox_Control';
								$cz_settings['sanitize_callback'] = 'GDLR_Core_Customize_Checkbox_Control::sanitize_customizer_checkbox';
								$cz_settings['sanitize_js_callback'] = 'GDLR_Core_Customize_Checkbox_Control::sanitize_js_customizer_checkbox';
							}else if( $option['type'] == 'fontslider' ){
								if( !empty($option['data-type']) && $option['data-type'] == 'opacity' ){
									$option['data-min'] = 0;
									$option['data-max'] = 100;
									$option['data-suffix'] = 'none';
								}

								$cz_control = 'GDLR_Core_Customize_FontSlider_Control';
								$cz_controls['choices'] = array();
								if( isset($option['data-min']) ){ $cz_controls['choices']['min'] = $option['data-min']; }
								if( isset($option['data-max']) ){ $cz_controls['choices']['max'] = $option['data-max']; }
								if( isset($option['data-suffix']) ){ $cz_controls['choices']['suffix'] = $option['data-suffix']; }
								$cz_controls['max'] = '99';
							}else if( $option['type'] == 'font' ){
								$this->google_font_link = true;
								$cz_control = 'GDLR_Core_Customize_Font_Control';
							}else if( $option['type'] == 'customizer-description' ){
								$cz_control = 'GDLR_Core_Customize_Description';
							}
							
							// assign variable for post script
							if( !empty($option['selector']) ){
								if( empty($option['data-type']) ){
									$option['data-type'] = 'color';
								}
								
								$cz_settings['transport'] = 'postMessage';
								$this->post_variable[] = array(
									'slug' => $option_slug,
									'selector' => $option['selector'],
									'data_type' => $option['data-type'],
									'control' => $main_section['slug'] . "[$option_slug]"
								);
							} 
		
							// adding the elements
							if( !empty($cz_control) ){
								$wp_customize->add_setting($main_section['slug'] . "[$option_slug]", array_merge(
									array('sanitize_callback'=>'gdlr_core_escape_content'), $cz_settings));
								$wp_customize->add_control(new $cz_control($wp_customize, $option_slug, $cz_controls));
							}
							
						} // option
					} // sub section
				} // main section

			} // register_customizer_option	
			
			// escape content with html
			function sanitize_customizer_media($file_url){
				global $wpdb;
		
				$attachment = $wpdb->get_col($wpdb->prepare("SELECT ID FROM {$wpdb->posts} WHERE guid='%s';", $file_url ) ); 
				if( !empty($attachment) ){
					return $attachment[0]; 
				}
				return $file_url;
			}
			
			// after the save event is triggered
			function after_save_theme_option(){
				$this->gdlr_core_admin_option->after_save_theme_option();
			}
	
		} // gdlr_core_theme_customizer
		
	} // class_exists