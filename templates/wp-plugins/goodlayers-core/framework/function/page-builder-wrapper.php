<?php
	/*	
	*	Goodlayers Item For Wrapper Section
	*/
	
	gdlr_core_page_builder::add_section('wrapper', 'gdlr_core_page_builder_wrapper');
	
	if( !class_exists('gdlr_core_page_builder_wrapper') ){
		class gdlr_core_page_builder_wrapper extends gdlr_core_page_builder_section{
				
			private static $pb_elements = array(
				'background' => 'gdlr_core_pb_wrapper_background',
				'sidebar' => 'gdlr_core_pb_wrapper_sidebar',
				'column' => 'gdlr_core_pb_wrapper_column',
				// 'flexbox' => 'gdlr_core_pb_wrapper_flexbox'
			);
			
			// use to init the element
			static function init(){
				add_action('wp_ajax_gdlr_core_get_pb_wrapper_preview', 'gdlr_core_page_builder_wrapper::get_wrapper_preview');
			}			
			
			// get the section settings
			static function get_settings(){
				return array(	
					'icon' => GDLR_CORE_URL . '/framework/images/page-builder/nav-wrapper.png', 
					'title' => esc_html__('Wrapper / Columns', 'goodlayers-core')
				);
			}			
			
			// assign the page builder variable
			static function set_page_builder_var( $page_builder_var = array() ){
				
				// assign column variable
				$column_settings = call_user_func(self::$pb_elements['column'] . '::get_settings');
				ksort($column_settings['list']);
				$page_builder_var['column'] = $column_settings['list'];
			
				// assign template
				foreach( self::$pb_elements as $wrapper_slug => $wrapper_class ){
					$page_builder_var['template'][$wrapper_slug] = call_user_func( $wrapper_class . '::get_template' );
				}
			
				return $page_builder_var;
			}
			
			// get the option of each registered element
			static function get_options(){
				if( empty($_POST['type']) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('An Error Occurs', 'goodlayers-core'),
						'message'=> esc_html__('Could not identify an option\'s type.' ,'goodlayers-core')
					)));
				}else{
					return call_user_func(self::$pb_elements[$_POST['type']] . '::get_options');
				}
			}

			// get element list for page builder nav bar
			static function get_element_list( $args = array() ){
				
				// for wrapper
				echo '<div class="gdlr-core-page-builder-head-content-wrapper-left">';
				
				// for background wrapper
				$settings = call_user_func(self::$pb_elements['background'] . '::get_settings');
				echo '<div class="gdlr-core-page-builder-head-content-wrapper-item gdlr-core-pb-list-draggable" data-template="wrapper" data-type="background" >';
				echo '<div class="gdlr-core-page-builder-head-content-wrapper-item-thumbnail gdlr-core-media-image" >';
				echo '<img src="' . esc_url($settings['thumbnail']) . '" alt="" />';
				echo '</div>';
				echo '<div class="gdlr-core-page-builder-head-content-wrapper-item-title" >' . gdlr_core_escape_content($settings['title']) . '</div>';
				echo '</div>';

				// for sidebar wrapper
				$settings = call_user_func(self::$pb_elements['sidebar'] . '::get_settings');
				echo '<div class="gdlr-core-page-builder-head-content-wrapper-item gdlr-core-pb-list-draggable" data-template="wrapper" data-type="sidebar" >';
				echo '<div class="gdlr-core-page-builder-head-content-wrapper-item-thumbnail gdlr-core-media-image" >';
				echo '<img src="' . esc_url($settings['thumbnail']) . '" alt="" />';
				echo '</div>';
				echo '<div class="gdlr-core-page-builder-head-content-wrapper-item-title" >' . gdlr_core_escape_content($settings['title']) . '</div>';
				echo '</div>';				
				
				echo '</div>'; // gdlr-core-page-builder-head-content-wrapper-left
				
				// for columns
				echo '<div class="gdlr-core-page-builder-head-content-wrapper-right">';
				$settings = call_user_func(self::$pb_elements['column'] . '::get_settings');
				foreach( $settings['list'] as $column_number => $column_size ){
					$image_url = $settings['image_dir'] . '/column-' . trim(str_replace('/', '-', $column_size)) . '.png';
					
					echo '<div class="gdlr-core-page-builder-head-content-wrapper-item gdlr-core-pb-list-draggable" data-template="wrapper" data-type="column" data-column="' . esc_attr($column_number) . '" >';
					echo '<div class="gdlr-core-page-builder-head-content-wrapper-item-thumbnail gdlr-core-media-image" >';
					echo '<img src="' . esc_url($image_url) . '" alt="" />';
					echo '</div>';
					echo '<div class="gdlr-core-page-builder-head-content-wrapper-item-title" >' . gdlr_core_escape_content($column_size) . '</div>';
					echo '</div>';
				}

				// for flexbox
				if( !empty(self::$pb_elements['flexbox']) ){
					$settings = call_user_func(self::$pb_elements['flexbox'] . '::get_settings');
					echo '<div class="gdlr-core-page-builder-head-content-wrapper-item gdlr-core-pb-list-draggable" data-template="wrapper" data-type="flexbox" >';
					echo '<div class="gdlr-core-page-builder-head-content-wrapper-item-thumbnail gdlr-core-media-image" >';
					echo '<img src="' . esc_url($settings['thumbnail']) . '" alt="" />';
					echo '</div>';
					echo '<div class="gdlr-core-page-builder-head-content-wrapper-item-title" >' . gdlr_core_escape_content($settings['title']) . '</div>';
					echo '</div>';
				}
				
				echo '<div class="clear"></div>';
				echo '</div>'; // gdlr-core-page-builder-head-content-wrapper-right
				
				echo '<div class="clear"></div>';
				
			}
			
			// get template for page builder
			static function get_template( $options = array(), $callback = '' ){
				if( !empty($options['sync-template']) ){
					$sync_options = gdlr_core_page_builder_sync_template::get_sync_template($options['sync-template']);
					if( !empty($sync_options) ){
						$options = $sync_options;
					}else{
						unset($options['sync-template']);
					}
				}

				return call_user_func(self::$pb_elements[$options['type']] . '::get_template', $options, $callback);
			}			
			static function get_content( $options = array(), $callback = '' ){
				if( !empty($options['sync-template']) ){
					$sync_options = gdlr_core_page_builder_sync_template::get_sync_template($options['sync-template']);
					if( !empty($sync_options) ){
						$options = $sync_options;
					}else{
						unset($options['sync-template']);
					}
				}
				
				return call_user_func(self::$pb_elements[$options['type']] . '::get_content', $options, $callback);
			}
			
			// get preview for ajax
			static function get_wrapper_preview(){
				
				if( !check_ajax_referer('gdlr_core_page_builder', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}
				
				if( !empty($_POST['type']) ){
					$value = empty($_POST['value'])? array(): gdlr_core_process_post_data($_POST['value']); 
					$value = gdlr_core_sanitize_input($value);
					if( is_array($value) && !empty($_POST['post_id']) ){
						global $page_builder_post_id;
						$page_builder_post_id = $_POST['post_id'];
					}
					$content = call_user_func(self::$pb_elements[$_POST['type']] . '::get_template', $value, 'gdlr_core_page_builder::get_page_builder_item');
				
					die(json_encode(array(
						'status' => 'success',
						'preview_content' => $content
					))); 
				}			
				
			}
			
		} // gdlr_core_page_builder_wrapper
	} // class_exists