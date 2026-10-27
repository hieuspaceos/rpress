<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('space', 'gdlr_core_pb_element_space'); 
	
	if( !class_exists('gdlr_core_pb_element_space') ){
		class gdlr_core_pb_element_space{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-arrows-v',
					'title' => esc_html__('Space', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'padding-bottom' => array(
								'title' => esc_html__('Height', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'description' => 'You can put minus value for pull the below content up. Eg. "-30px"',
								'default' => $gdlr_core_item_pdb
							),
						)
					),
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings);
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array() ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$extra_class  = empty($settings['class'])? '': $settings['class'];
				$ret  = '<div class="gdlr-core-space-item gdlr-core-item-pdlr ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) ){
					$padding_bottom = intval(str_replace('px', '', $settings['padding-bottom']));

					if( $padding_bottom < 0 ){
						$ret .= gdlr_core_esc_style(array('margin-top'=>$settings['padding-bottom']));
					}else{
						$ret .= gdlr_core_esc_style(array('padding-top'=>$settings['padding-bottom']));
					}
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' ></div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_space
	} // class_exists	

	// [gdlr_core_space height="30px"]
	add_shortcode('gdlr_core_space', 'gdlr_core_space_shortcode');
	if( !function_exists('gdlr_core_space_shortcode') ){
		function gdlr_core_space_shortcode($atts, $content = ''){
			$atts = shortcode_atts(array(
				'height' => '30px', 
			), $atts, 'gdlr_core_space');

			return '<span class="gdlr-core-space-shortcode" ' . gdlr_core_esc_style(array(
				'margin-top' => $atts['height']
			)) . ' >' . gdlr_core_escape_content($content) . '</span>';
		}
	}	