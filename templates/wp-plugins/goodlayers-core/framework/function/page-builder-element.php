<?php
	/*	
	*	Goodlayers Item For Wrapper Section
	*/
	
	gdlr_core_page_builder::add_section('element', 'gdlr_core_page_builder_element');
	
	if( !class_exists('gdlr_core_page_builder_element') ){
		class gdlr_core_page_builder_element extends gdlr_core_page_builder_section{
			
			// static function for elements registration
			private static $pb_elements = array();
			static function add_element( $slug, $class_name ){
				self::$pb_elements[$slug] = $class_name;
			}
			
			// use to init the element
			static function init(){
				add_action('after_setup_theme', 'gdlr_core_page_builder_element::sort_pb_elements', 999);
				add_action('wp_ajax_gdlr_core_get_pb_element_preview', 'gdlr_core_page_builder_element::get_element_preview');
			}
			static function sort_pb_elements(){
				ksort(self::$pb_elements);
			}

			// get the section settings
			static function get_settings(){	
				return array(	
					'icon' => GDLR_CORE_URL . '/framework/images/page-builder/nav-elements.png', 
					'title' => esc_html__('Elements', 'goodlayers-core')
				);
			}
			
			// assign the page builder variable
			static function set_page_builder_var( $page_builder_var = array() ){
				
				$page_builder_var['template']['element'] = self::get_template(); 
				
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
				
				// search
				echo '<input type="text" placeholder="' . esc_attr('Search Elements', 'goodlayers-core') . '" class="gdlr-core-page-builder-head-content-search" />';
				
				// elements
				echo '<div class="gdlr-core-page-builder-head-content-element-container" >';
				
				foreach( self::$pb_elements as $element_slug => $element_class ){
					$settings = call_user_func( $element_class . '::get_settings' );

					if( !empty($args['element-type']) ){
						if( empty($settings['type']) || !in_array($args['element-type'], $settings['type']) ){
							continue;
						}
					}else{
						if( !empty($settings['type']) && !in_array('default', $settings['type']) ){
							continue;
						}
					}
					
					echo '<div class="gdlr-core-page-builder-head-content-element-item gdlr-core-pb-list-draggable" ';
					echo 'data-template="element" data-type="' . esc_attr($element_slug) . '" data-title="' . esc_attr($settings['title']) . '" >';
					if( !empty($settings['icon']) ){
						echo '<div class="gdlr-core-page-builder-head-content-element-item-icon" >';
						if( strpos($settings['icon'], 'fa-') === 0 ){
							echo '<i class="fa ' . esc_attr($settings['icon']) . '" ></i>';
						}else{
							echo '<i class="' . esc_attr($settings['icon']) . '" ></i>';
						}
						echo '</div>';
					}
					if( !empty($settings['title']) ){
						echo '<div class="gdlr-core-page-builder-head-content-element-item-title" >' . gdlr_core_escape_content($settings['title']) . '</div>';
					}
					echo '</div>';	
				}

				echo '<div class="clear"></div>';
				echo '</div>'; // gdlr-core-page-builder-head-content-elements-container
				
			}
			
			// get template for page builder
			static function get_template( $options = array(), $callback = ''){
				
				if( !empty($options['type']) && !empty(self::$pb_elements[$options['type']]) ){
					$e_settings = call_user_func(self::$pb_elements[$options['type']] . '::get_settings');
				}
				
				$element  = '<div class="gdlr-core-page-builder-item" data-template="element" ';
				$element .= (empty($options['type'])? '': 'data-type="' . esc_attr($options['type']) . '" ');
				$element .= (empty($options['value'])? '': 'data-value="' . esc_attr(htmlspecialchars(json_encode($options['value']))) . '"');
				$element .= '>';
				$element .= '<div class="gdlr-core-page-builder-item-content">';
				$element .= '<div class="gdlr-core-page-builder-item-head clearfix">';
				$element .= '<span class="gdlr-core-page-builder-item-head-item-title">' . (empty($e_settings['title'])? '': gdlr_core_escape_content($e_settings['title'])) . '</span>';
				$element .= '<i class="fa fa-remove gdlr-core-page-builder-item-remove" ></i>';
				$element .= '<i class="fa fa-copy gdlr-core-page-builder-item-copy" ></i>';
				$element .= '<i class="fa fa-download gdlr-core-page-builder-item-save" ></i>';
				$element .= '<i class="fa fa-edit gdlr-core-page-builder-item-edit" ></i>';
				$element .= '</div>';
				$element .= '<div class="gdlr-core-page-builder-item-container">';
				$element .= '<div class="gdlr-core-page-builder-item-container-item">';
				$element .= '<i class="fa fa-align-justify gdlr-core-page-builder-item-container-item-icon"></i>';
				$element .= '<span class="gdlr-core-page-builder-item-container-item-title">' . (empty($e_settings['title'])? '': gdlr_core_escape_content($e_settings['title'])) . '</span>';
				$element .= '</div>';
				$element .= '<div class="gdlr-core-page-builder-item-container-preview">';
				if( !empty($options['type']) && !empty(self::$pb_elements[$options['type']]) ){
					$options['value'] = empty($options['value'])? '': $options['value'];
					if( !defined('GDLR_CORE_DEBUG') ){
						$element .= call_user_func(self::$pb_elements[$options['type']] . '::get_preview', $options['value']);
					}
				}

				$element .= '<div class="gdlr-core-page-builder-item-container-overlay"></div>';
				$element .= '</div>';
				$element .= '<i class="fa fa-edit gdlr-core-page-builder-item-edit" ></i>';
				$element .= '</div>'; // gdlr-core-page-builder-item-container
				$element .= '</div>'; // gdlr-core-page-builder-item-content
				$element .= '</div>'; // gdlr-core-page-builder-item

				return $element;
			}
			
			// get element content for front end page builder
			static function get_content( $options = array(), $callback = '' ){
				
				$element = '';
				if( !empty($options['type']) && !empty(self::$pb_elements[$options['type']]) ){
					$settings = call_user_func(self::$pb_elements[$options['type']] . '::get_settings' );
					$pb_item_wrap = empty($settings['nowrap']);

					$options['value'] = empty($options['value'])? '': $options['value'];
					
					$element  = $pb_item_wrap? '<div class="gdlr-core-pbf-element" >' : '';
					$element .= call_user_func(self::$pb_elements[$options['type']] . '::get_content', $options['value']);
					$element .= $pb_item_wrap? '</div>': '';
				}
				
				return $element;
			}
			
			// get preview for page builder
			static function get_element_preview(){
				
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
					$content = call_user_func(self::$pb_elements[$_POST['type']] . '::get_preview', $value);
				
					die(json_encode(array(
						'status' => 'success',
						'preview_content' => $content
					))); 
				}
				
			} // get_element_preview
			
		} // gdlr_core_page_builder_element
	} // class_exists