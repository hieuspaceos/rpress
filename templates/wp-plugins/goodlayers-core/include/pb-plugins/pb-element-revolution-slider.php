<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('revolution-slider', 'gdlr_core_pb_element_revolution_slider'); 
	
	if( !class_exists('gdlr_core_pb_element_revolution_slider') ){
		class gdlr_core_pb_element_revolution_slider{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-image',
					'title' => esc_html__('Revolution Slider', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'revolution-slider-id' => array(
								'title' => esc_html__('Choose Revolution Slider', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => gdlr_core_get_revolution_slider_list()
							),
							'z-index' => array(
								'title' => esc_html__('Z Index', 'goodlayers-core'),
								'type' => 'text'
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
								'default' => '0px'
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
						'revolution-slider-id' => '',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$extra_class = empty($settings['class'])? '': $settings['class'];
				$ret  = '<div class="gdlr-core-revolution-slider-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				$ret .= gdlr_core_esc_style(array(
					'padding-bottom' => (!empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb)? $settings['padding-bottom']: '',
					'z-index' => empty($settings['z-index'])? '': $settings['z-index']
				));
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				
				// display
				if( !class_exists('RevSliderSlider') ){
					$message = esc_html__('Please install and activate the "Revolution Slider" plugin to show the slides.', 'goodlayers-core');
				}else if( empty($settings['revolution-slider-id']) ){
					$message = esc_html__('Please select the id of the revolution slider you created.', 'goodlayers-core');
				}else if( $preview ){
					$message = '[rev_slider alias="' . esc_attr($settings['revolution-slider-id']) . '"]';
				}else{
					$rev_slider = gdlr_core_get_revolution_slider_list();
					if( !empty($rev_slider[$settings['revolution-slider-id']]) ){
						$ret .= do_shortcode('[rev_slider alias="' . esc_attr($settings['revolution-slider-id']) . '"]');
					}else{
						$message  = 'Slide not found. Please create the revolution slider then coose the slider ID when you edit this page.<br>';
						$message .= '1.) If you want to import slider demo content, please watch this video <a href="https://www.youtube.com/watch?v=3YmMFTXiWZU&feature=youtu.be&t=52s" target="_blank">https://www.youtube.com/watch?v=3YmMFTXiWZU&feature=youtu.be&t=52s</a> from 0:52 sec.<br>';
						$message .= '2.) Edit this page and select the proper slide from slider item.';
					}
				}
				if( !empty($message) ){
					$ret .= '<div class="gdlr-core-external-plugin-message">' . gdlr_core_escape_content($message) . '</div>';
				}
				
				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_revolution_slider
	} // class_exists	

	// get slider list
	if( !function_exists('gdlr_core_get_revolution_slider_list') ){
		function gdlr_core_get_revolution_slider_list(){
			if( !class_exists('RevSliderSlider') ) return array();

			$slider = new RevSliderSlider();
			if( method_exists($slider, 'getArrSlidersShort') ){
				return $slider->getArrSlidersShort();
			}else if( method_exists($slider, 'get_sliders_short_list') ){
				return $slider->get_sliders_short_list();
			}
				
			return array();
		}
	}