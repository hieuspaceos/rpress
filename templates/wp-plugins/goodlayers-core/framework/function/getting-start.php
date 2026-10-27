<?php
	/*	
	*	Goodlayers Getting Start
	*	---------------------------------------------------------------------
	*	This file contains function for getting start page
	*	---------------------------------------------------------------------
	*/
	
	if( !class_exists('gdlr_core_getting_start') ){
		class gdlr_core_getting_start{

			private $settings;

			function __construct( $settings = array() ){

				$this->settings = wp_parse_args($settings, array(
					'parent-slug' => '',
					'page-title' => esc_html__('Goodlayers Option', 'goodlayers-core'),
					'menu-title' => esc_html__('Goodlayers Option', 'goodlayers-core'),
					'capability' => 'edit_theme_options',
					'slug' => 'gdlr_core_getting_start'
				));

				// add action to create dashboard
				if( class_exists('gdlr_core_admin_menu') ){
					
					gdlr_core_admin_menu::register_menu(array(
						'parent-slug' => $this->settings['parent-slug'],
						'page-title' => $this->settings['page-title'], 
						'menu-title' =>$this->settings['menu-title'], 
						'capability' => $this->settings['capability'], 
						'menu-slug' => $this->settings['slug'], 
						'function' => array(&$this, 'getting_start_content')
					));
					
				}

				// enqueue script for getting start page
				add_action('admin_enqueue_scripts', array(&$this, 'enqueue_script'));

			}

			// script for getting start page
			function enqueue_script($hook){
				if( strpos($hook, 'page_' . $this->settings['slug']) !== false ){
						
					gdlr_core_include_utility_script();

					// include the admin style
					gdlr_core_include_icon_font('font-awesome');
					wp_enqueue_style('gdlr-core-getting-start', GDLR_CORE_URL . '/framework/css/getting-start.css');
					wp_enqueue_style('open-sans-css', 'https://fonts.googleapis.com/css?family=Open+Sans:400,300,300italic,400italic,600,600italic,700,700italic,800,800italic&subset=latin,latin-ext');
					
					// include the admin script
					wp_enqueue_script('gdlr-core-getting-start', GDLR_CORE_URL . '/framework/js/getting-start.js', array('jquery'), false, true);
					wp_localize_script('gdlr-core-getting-start', 'gdlr_core_ajax_message', array(
						'ajaxurl' => GDLR_CORE_AJAX_URL,
						'error_head' => esc_html__('An error occurs', 'goodlayers-core'),
						'error_message' => esc_html__('Please try again. If the problem still persists, please contact administrator for this.', 'goodlayers-core'),
						'nonce' => wp_create_nonce('gdlr_core_demo_import'),

						'importing_head' => esc_html__('Importing demo content. Please wait...', 'goodlayers-core'),
						'importing_content' => esc_html__('If you choose to download images from demo site, it can take up to 7-8 minutes so please be patient.', 'goodlayers-core'),
					));	
				}				
			}	

			// generate content for getting start page
			function getting_start_content(){

				$options = apply_filters('gdlr_core_getting_start_option', array(), $this->settings['slug']);

				echo '<div class="gdlr-core-getting-start-wrap clearfix" >';
				$this->get_header($options['header']);

				$this->get_content($options['content']);
				echo '</div>'; // gdlr-core-getting-start-wrap

				if( isset($_GET['phpinfo']) ) print_r( phpinfo() );
			}

			// header section
			function get_header( $options ){
				echo '<div class="gdlr-core-getting-start-header clearfix" >';
				if( !empty($options['logo']) ){ 
					echo '<div class="gdlr-core-getting-start-header-image" >';
					echo gdlr_core_get_image($options['logo']);

					$theme_info = wp_get_theme();
					echo '<div class="gdlr-core-getting-start-header-info">';
					echo '<span class="gdlr-core-head" >' . $theme_info->get('Name') . '</span>';
					echo '<span class="gdlr-core-sep" ></span>';
					echo '<span class="gdlr-core-tail" >VER. ' . $theme_info->get('Version') . '</span>';
					echo '</div>';
					echo '</div>';
				}
				echo '<div class="gdlr-core-getting-start-header-content" >';
				if( !empty($options['title']) ){ 
					echo '<h3 class="gdlr-core-getting-start-header-title" >' . $options['title'] . '</h3>';
				}	
				if( !empty($options['caption']) ){ 
					echo '<div class="gdlr-core-getting-start-header-caption" >' . $options['caption'] . '</div>';
				}
				echo '</div>';
				echo '</div>'; // gdlr-core-getting-start-header
			}

			// content section
			function get_content( $options ){
				echo '<div class="gdlr-core-getting-start-content-wrap clearfix" >';

				// nav bar
				$has_active = false;
				echo '<div class="gdlr-core-getting-start-nav" id="gdlr-core-getting-start-nav" >';
				foreach( $options as $slug => $option ){
					if( !empty($option) ){
						echo '<a ';
						if( empty($has_active) && $option['type'] != 'link' ){
							echo ' class="gdlr-core-active" ';
							$has_active = true;
						}
						switch( $option['type'] ){
							case 'link': 
								echo 'href="' . esc_url($option['url']) . '" ';
								echo empty($option['target'])? 'target="_self" ': 'target="' . esc_attr($option['target']) . '" ';
								break;
							default :
								echo 'href="#" data-page="' . esc_attr($slug) . '" ';
						}
						echo ' >' . $option['title'] . '</a>';
					}
				}
				echo '</div>';

				// nav content
				$has_active = false;
				echo '<div class="gdlr-core-getting-start-content" id="gdlr-core-getting-start-content" >';
				foreach( $options as $slug => $option ){
					if( !empty($option) && $option['type'] != 'link' ){
						echo '<div class="gdlr-core-getting-start-page ' . (!$has_active? 'gdlr-core-active': '') . '" data-page="' . esc_attr($slug) . '" >';
						if( !empty($option['content']) ){
							echo '<div class="gdlr-core-getting-start-page-content" >';
							echo gdlr_core_escape_content($option['content']);
							echo '</div>';
						}
						switch( $option['type'] ){
							case 'demo': 
								$content = empty($option['demo-content'])? '': $option['demo-content'];
								$this->get_demo_import($content);
								break;
							case 'system-status': 
								$this->get_system_status();
								break;
						}
						echo '</div>';

						$has_active = true;
					}
				}
				echo '</div>';

				echo '</div>'; // gdlr-core-getting-start-content-wrap
			}

			// demo import
			function get_demo_import($content){
				echo '<div class="gdlr-core-demo-import-wrap clearfix" id="gdlr-core-demo-import-form" >';

				echo '<div class="gdlr-core-demo-import-success" id="gdlr-core-demo-import-success" ></div>';

				// first
				echo '<div class="gdlr-core-demo-import-section-wrap clearfix" >';
				echo '<div class="gdlr-core-demo-import-section-head" >';
				echo '<span class="gdlr-core-steps">1</span>';
				echo '<span class="gdlr-core-head">' . esc_html__('Select Demo', 'goodlayers-core') . '</span>';
				echo '</div>';

				$demo_options = apply_filters('gdlr_core_demo_options', array());

				$first_url = '';
				echo '<div class="gdlr-core-demo-import-list">';
				echo '<div class="gdlr-core-demo-import-combobox" >';
				echo '<select data-name="demo-id" id="gdlr-core-demo-import-option" >';
				foreach( $demo_options as $option_slug => $options ){
					echo '<option value="' . esc_attr($option_slug) . '" data-url="' . esc_url($options['url']) . '" >' . $options['title'] . '</option>';
					$first_url = empty($first_url)? $options['url']: $first_url;
				}	
				echo '</select>';
				echo '</div>';

				echo '<a href="' . esc_url($first_url) . '" class="gdlr-core-view-demo-button" id="gdlr-core-view-demo-button" target="_blank">' . esc_html__('View Demo', 'goodlayers-core') . '<i class="fa fa-external-link" ></i></a>';
				
				echo '<div class="gdlr-core-demo-import-list-content" >' . $content . '</div>';
				echo '</div>'; // gdlr-core-demo-import-list
				echo '</div>'; // gdlr-core-demo-import-section-wrap

				// second
				echo '<div class="gdlr-core-demo-import-section-wrap clearfix" >';
				echo '<div class="gdlr-core-demo-import-section-head" >';
				echo '<span class="gdlr-core-steps">2</span>';
				echo '<span class="gdlr-core-head">' . esc_html__('Import Settings', 'goodlayers-core') . '</span>';
				echo '</div>';
				echo '<div class="gdlr-core-demo-import-section-option" >';

				echo '<div class="gdlr-core-demo-import-option" >';
				echo '<input type="checkbox" data-name="navigation" checked >';
				echo '<span class="gdlr-core-option-text" >' . esc_html__('Include menu navigation', 'goodlayers-core') . '</span>';
				echo '</div>';

				echo '<div class="gdlr-core-demo-import-option" >';
				echo '<input type="checkbox" data-name="post" checked >';
				echo '<span class="gdlr-core-option-text" >' . esc_html__('Include blog posts content from the demo', 'goodlayers-core') . '</span>';
				echo '</div>';

				echo '<div class="gdlr-core-demo-import-option" >';
				echo '<input type="checkbox" data-name="portfolio" checked >';
				echo '<span class="gdlr-core-option-text" >' . esc_html__('Include portfolio posts from the demo', 'goodlayers-core') . '</span>';
				echo '</div>';

				echo '<div class="gdlr-core-demo-import-option" >';
				echo '<input type="checkbox" data-name="image" checked >';
				echo '<span class="gdlr-core-option-text" >' . esc_html__('Download images from demo site', 'goodlayers-core') . ' ( <a href="#" id="gdlr-core-image-condition" >' . esc_html__('read conditions', 'goodlayers-core') . '</a> )</span>';
				echo '<div class="gdlr-core-image-condition-wrap" id="gdlr-core-image-condition-wrap" >';
				echo '<div class="gdlr-core-image-condition-content" >';
				echo esc_html__('Some of the images used on the demo site are licensed, so if you plan to use them in your final product, ensure that you purchase the proper license. The image links are included in the main package you downloaded from Themeforest, located in the folder \'Demo Stuffs > Image Links\' (Note that the image files themselves are not directly included in the theme package). However, some licensed images may not be listed as they have been removed from the stock photos website. Conversely, some unlisted images may be free to use as they fall under the CC0 license. '); 
				echo '<br><br>';
				echo esc_html__('Therefore, we encourage you to replace the images with your own or delete unused images from the Media section. Otherwise, make sure to purchase the appropriate license from the list we provided. If you are unsure about specific images, feel free to contact us for assistance.', 'goodlayers-core');
				echo '</div>';
				echo '<div class="gdlr-core-condition-close" ><i class="fa fa-remove" ></i>' . esc_html__('Close', 'goodlayers-core') . '</div>';
				echo '</div>';
				echo '</div>';

				echo '<div class="gdlr-core-demo-import-option" >';
				echo '<input type="checkbox" data-name="theme-option" checked >';
				echo '<span class="gdlr-core-option-text" >' . esc_html__('Import theme options', 'goodlayers-core') . ' ( <span class="gdlr-core-red">' . esc_html__('Noted that the current theme option will be overridden', 'goodlayers-core') . '</span> )</span>';
				echo '</div>';

				echo '<div class="gdlr-core-demo-import-option" >';
				echo '<input type="checkbox" data-name="widget" checked >';
				echo '<span class="gdlr-core-option-text" >' . esc_html__('Import widget (sidebar & footer)', 'goodlayers-core') . '</span>';
				echo '</div>';

				echo '<a class="gdlr-core-demo-import-button" id="gdlr-core-demo-import-submit" >' . esc_html__('Start Import Demo!', 'goodlayers-core') . '</a>';
				echo '</div>';
				echo '</div>'; // gdlr-core-demo-import-section-wrap

				echo '</div>'; // gdlr-core-demo-import-wrap
			}

			// system status
			function get_system_status(){
				echo '<div class="gdlr-core-system-status-wrap" >';
				echo '<div class="gdlr-core-system-status-head" >' . esc_html__('System Status', 'goodlayers-core') . '</div>';
				echo '<table><tbody>';

				// debug
				echo '<tr>';
				echo '<td class="gdlr-core-table-head" >' . esc_html__('Debug Mode', 'goodlayers-core') . '</td>';
				echo '<td class="gdlr-core-table-content" >';
				if( defined('WP_DEBUG') && WP_DEBUG ){
					echo esc_html__('On', 'goodlayers-core');
					echo '<span class="gdlr-core-recommendation">' . esc_html__('You should turn debug mode off when you make your site live', 'goodlayers-core') . '</div>';
				}else{
					echo esc_html__('Off', 'goodlayers-core');
				}
				echo '</td>';
				echo '</tr>';

				// php version
				echo '<tr>';
				echo '<td class="gdlr-core-table-head" >' . esc_html__('PHP Version', 'goodlayers-core') . '</td>';
				echo '<td class="gdlr-core-table-content" >';
				if( function_exists('phpversion') ){
					echo phpversion();
				}else{
					echo '-';
				}
				echo '</td>';
				echo '</tr>';

				// wp upload max size
				echo '<tr>';
				echo '<td class="gdlr-core-table-head" >' . esc_html__('wp_max_upload_size', 'goodlayers-core') . '</td>';
				echo '<td class="gdlr-core-table-content" >';
				$wp_max_upload_size = wp_max_upload_size();
				echo intval($wp_max_upload_size / 1048576) . 'M';
				echo '</td>';
				echo '</tr>';

				if( function_exists('ini_get') ){

					// upload max size
					echo '<tr>';
					echo '<td class="gdlr-core-table-head" >' . esc_html__('upload_max_filesize', 'goodlayers-core') . '</td>';
					echo '<td class="gdlr-core-table-content" >';
					echo ini_get('upload_max_filesize');
					echo '</td>';
					echo '</tr>';

					// post max size
					echo '<tr>';
					echo '<td class="gdlr-core-table-head" >' . esc_html__('post_max_size', 'goodlayers-core') . '</td>';
					echo '<td class="gdlr-core-table-content" >';
					echo ini_get('post_max_size');
					echo '</td>';
					echo '</tr>';

					// max execution time
					echo '<tr>';
					echo '<td class="gdlr-core-table-head" >' . esc_html__('max_execution_time', 'goodlayers-core') . '</td>';
					echo '<td class="gdlr-core-table-content" >';
					$max_execution_time = ini_get('max_execution_time');
					echo gdlr_core_escape_content($max_execution_time);
					if( $max_execution_time < 300 ){
						echo '<span class="gdlr-core-recommendation">' . esc_html__('Recommend to be over 300 for demo import process', 'goodlayers-core') . '</div>';
					}
					echo '</td>';
					echo '</tr>';

					// memory limit
					echo '<tr>';
					echo '<td class="gdlr-core-table-head" >' . esc_html__('memory_limit', 'goodlayers-core') . '</td>';
					echo '<td class="gdlr-core-table-content" >';
					$memory_limit = ini_get('memory_limit');
					echo gdlr_core_escape_content($memory_limit);
					$memory_limit = intval(str_replace('M', '', $memory_limit));
					if( $memory_limit < 512 ){
						echo '<span class="gdlr-core-recommendation">' . esc_html__('Recommend to be 512M for demo import process', 'goodlayers-core') . '</div>';
					}
					echo '</td>';
					echo '</tr>';

					// max input var
					echo '<tr>';
					echo '<td class="gdlr-core-table-head" >' . esc_html__('max_input_vars', 'goodlayers-core') . '</td>';
					echo '<td class="gdlr-core-table-content" >';
					$max_input_vars = ini_get('max_input_vars');
					echo gdlr_core_escape_content($max_input_vars);
					if( $max_input_vars < 4000 ){
						echo '<span class="gdlr-core-recommendation">' . esc_html__('Recommend value is 4000 for full demo import process', 'goodlayers-core') . '</div>';
					}
					echo '</td>';
					echo '</tr>';

					// default_socket_timeout
					echo '<tr>';
					echo '<td class="gdlr-core-table-head" >' . esc_html__('default_socket_timeout', 'goodlayers-core') . '</td>';
					echo '<td class="gdlr-core-table-content" >';
					$default_socket_timeout = ini_get('default_socket_timeout');
					echo gdlr_core_escape_content($default_socket_timeout);
					// if( $default_socket_timeout < 300 ){
					// 	echo '<span class="gdlr-core-recommendation">' . esc_html__('Recommend value to be over 300 for images import process', 'goodlayers-core') . '</div>';
					// }
					echo '</td>';
					echo '</tr>';

					// suhosin
					echo '<tr>';
					echo '<td class="gdlr-core-table-head" >' . esc_html__('suhosin', 'goodlayers-core') . '</td>';
					echo '<td class="gdlr-core-table-content" >';
					if( extension_loaded( 'suhosin' ) ){
						echo esc_html__('On', 'goodlayers-core');
					}else{
						echo esc_html__('Off', 'goodlayers-core');
					}
					echo '</td>';
					echo '</tr>';
					
					if( extension_loaded( 'suhosin' ) ){
						
						// post max var
						echo '<tr>';
						echo '<td class="gdlr-core-table-head" >' . esc_html__('suhosin.post.max_vars', 'goodlayers-core') . '</td>';
						echo '<td class="gdlr-core-table-content" >';
						echo ini_get('suhosin.post.max_vars');
						echo '</td>';
						echo '</tr>';

						// request max var
						echo '<tr>';
						echo '<td class="gdlr-core-table-head" >' . esc_html__('suhosin.request.max_vars', 'goodlayers-core') . '</td>';
						echo '<td class="gdlr-core-table-content" >';
						echo ini_get('suhosin.request.max_vars');
						echo '</td>';
						echo '</tr>';
					}

				}

				echo '</tbody></table>';

				echo '<div class="gdlr-core-system-status-footer" >' . esc_html__('You can change these values by editing php.ini file directly. Or you can ask your hosting provider to do it for you.', 'goodlayers-core') . '</div>';
				echo '</div>';
			}

		} // gdlr_core_getting_start
	} // class_exists
