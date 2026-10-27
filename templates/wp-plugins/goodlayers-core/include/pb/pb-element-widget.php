<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('widget', 'gdlr_core_pb_element_widget'); 
	
	if( !class_exists('gdlr_core_pb_element_widget') ){
		class gdlr_core_pb_element_widget{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-square',
					'title' => esc_html__('Widget', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'widget-id' => array(
								'title' => esc_html__('Select Widget Area', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => gdlr_core_sidebar_generator::get_sidebars()
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
								'default' => $gdlr_core_item_pdb
							)
						)
					)
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
						'widget-id' => '',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$extra_class  = empty($settings['class'])? '': $settings['class'];
				$ret  = '<div class="gdlr-core-widget-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( !empty($settings['widget-id']) && is_active_sidebar($settings['widget-id']) ){
					if( is_admin() ){
						$ret .= '<div class="gdlr-core-external-plugin-message">"' . $settings['widget-id']  . '" Widget</div>';
					}else{
						ob_start();
						dynamic_sidebar($settings['widget-id']);
						$ret .= ob_get_contents();
						ob_end_clean();	
					}
				}

				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_widget
	} // class_exists	