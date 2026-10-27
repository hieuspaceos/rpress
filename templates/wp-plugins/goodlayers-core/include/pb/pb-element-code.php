<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('code', 'gdlr_core_pb_element_code'); 
	
	if( !class_exists('gdlr_core_pb_element_code') ){
		class gdlr_core_pb_element_code{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-code',
					'title' => esc_html__('Code', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'content' => array(
								'title' => esc_html__('Content', 'goodlayers-core'),
								'type' => 'textarea',
								'default' => esc_html__('Code item sample content', 'goodlayers-core'),
								'wrapper-class' => 'gdlr-core-fullsize'
							),							
						)
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'style' => array(
								'title' => esc_html__('Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'light' => esc_html__('Light', 'goodlayers-core'),
									'dark' => esc_html__('Dark', 'goodlayers-core'),
								),
								'default' => 'light'
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom ( Item )', 'goodlayers-core'),
								'type' => 'text',
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
						'content' => esc_html__('Code item sample content', 'goodlayers-core'),
						'style' => 'light',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$extra_class  = ' gdlr-core-code-item-' . $settings['style'];
				$extra_class .= empty($settings['no-pdlr'])? ' gdlr-core-item-pdlr': '';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
 				$ret  = '<div class="gdlr-core-code-item gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				if( !empty($settings['content']) ){
					$ret .= '<div class="gdlr-core-code-item-content gdlr-core-content-font gdlr-core-skin-e-background gdlr-core-skin-e-content gdlr-core-skin-border">';
					$ret .= wpautop(htmlspecialchars($settings['content']));
					$ret .= '</div>';
				}
				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_code
	} // class_exists	

	add_shortcode('gdlr_core_code', 'gdlr_core_code_shortcode');
	add_shortcode('gdlr_core_code2', 'gdlr_core_code_shortcode');
	if( !function_exists('gdlr_core_code_shortcode') ){
		function gdlr_core_code_shortcode($atts, $content = ''){
			$atts = shortcode_atts(array(
				'style' => 'light',
				'padding-bottom' => '',
				'no-pdlr' => true
			), $atts, 'gdlr_core_code');
			
			$content = preg_replace('/\<br.*\>/', '', $content);
			$content = str_replace('<p>', '', $content);
			$content = str_replace('</p>', '', $content);
			$atts['content'] = $content;

			return gdlr_core_pb_element_code::get_content($atts);
		}
	}	