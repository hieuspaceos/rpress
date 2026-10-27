<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('master-slider', 'gdlr_core_pb_element_master_slider'); 
	
	if( !class_exists('gdlr_core_pb_element_master_slider') ){
		class gdlr_core_pb_element_master_slider{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-image',
					'title' => esc_html__('Master Slider', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'master-slider-id' => array(
								'title' => esc_html__('Choose Master Slider', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => gdlr_core_get_masterslider_list()
							)				
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom ( Item )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => 0
							)
						)
					)
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);			
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'master-slider-id' => '',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$extra_class = empty($settings['class'])? '': $settings['class'];
				$ret  = '<div class="gdlr-core-master-slider-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				
				// display
				if( !function_exists('get_masterslider') ){
					$message = esc_html__('Please install and activate the "Master Slider" plugin to show the slides.', 'goodlayers-core');
				}else if( empty($settings['master-slider-id']) ){
					$message = esc_html__('Please select the id of the master slider you created.', 'goodlayers-core');
				}else if( $preview ){
					$message = '[masterslider id="' . esc_attr($settings['master-slider-id']) . '"]';
				}else{
					$ret .= get_masterslider($settings['master-slider-id']);
				}
				if( !empty($message) ){
					$ret .= '<div class="gdlr-core-external-plugin-message">' . gdlr_core_escape_content($message) . '</div>';
				}
				
				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_master_slider
	} // class_exists	

	// get slider list
	if( !function_exists('gdlr_core_get_masterslider_list') ){
		function gdlr_core_get_masterslider_list(){
			if( !function_exists('get_masterslider_names') ) return array();
		
			return get_masterslider_names(true);
		}
	}