<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/

	add_action('plugins_loaded', 'gdlr_core_cpt_support_add_pb_element');
	if( !function_exists('gdlr_core_cpt_support_add_pb_element') ){
		function gdlr_core_cpt_support_add_pb_element(){

			if( class_exists('gdlr_core_page_builder_element') ){
				gdlr_core_page_builder_element::add_element('custom-field', 'gdlr_core_pb_element_custom_field'); 
			}
			
		}
	}
	
	if( !class_exists('gdlr_core_pb_element_custom_field') ){
		class gdlr_core_pb_element_custom_field{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-align-justify',
					'title' => esc_html__('Custom Field', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'custom-field-slug' => array(
								'title' => esc_html__('Custom Field Slug', 'goodlayers-core'),
								'type' => 'text',
								'wrapper-class' => 'gdlr-core-fullsize'
							),		
							'text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'left'
							),	
						)
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'font-size' => array(
								'title' => esc_html__('Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '',
								'description' => esc_html__('Leaving this field blank will display the default font size from theme options', 'goodlayers-core'),
							),
							'content-line-height' => array(
								'title' => esc_html__('Line Height', 'goodlayers-core'),
								'type' => 'text',
							),
							'content-font-weight' => array(
								'title' => esc_html__('Content Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'content-letter-spacing' => array(
								'title' => esc_html__('Content Font Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'content-text-transform' => array(
								'title' => esc_html__('Content Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => 'none'
							),
							'tablet-font-size' => array(
								'title' => esc_html__('Tablet Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'mobile-font-size' => array(
								'title' => esc_html__('Mobile Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
						)
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'text-color' => array(
								'title' => esc_html__('Text Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'margin-left' => array(
								'title' => esc_html__('Margin Left', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'margin-right' => array(
								'title' => esc_html__('Margin Right', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
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
				$content  = self::get_content($settings, true);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-text-box-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-text-box-<?php echo esc_attr($id); ?>').parent().gdlr_core_content_script();
});
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'custom-field-slug' => '',
						'text-align' => 'left',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				$custom_style = '';
				if( !empty($settings['tablet-font-size']) ){
					$custom_style .= '@media only screen and (max-width: 999px){';
					$custom_style .= '#custom_style_id .gdlr-core-text-box-item-content{ ' . gdlr_core_esc_style(array(
						'font-size' => $settings['tablet-font-size']
					), false, true) . ' }';
					$custom_style .= '}';
				}
				if( !empty($settings['mobile-font-size']) ){
					$custom_style .= '@media only screen and (max-width: 767px){';
					$custom_style .= '#custom_style_id .gdlr-core-text-box-item-content{ ' . gdlr_core_esc_style(array(
						'font-size' => $settings['mobile-font-size']
					), false, true) . ' }';
					$custom_style .= '}';
				}
				if( !empty($custom_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_custom_field_id;

						if( $preview ){
							$gdlr_core_custom_field_id = empty($gdlr_core_custom_field_id)? array(): $gdlr_core_gdlr_core_custom_field_idtext_box_id;

							// generate unique id so it does not get overwritten in admin area
							$rnd_custom_field_id = mt_rand(0, 99999);
							while( in_array($rnd_custom_field_id, $gdlr_core_custom_field_id) ){
								$rnd_custom_field_id = mt_rand(0, 99999);
							}
							$gdlr_core_custom_field_id[] = $rnd_custom_field_id;
							$settings['id'] = 'gdlr-core-custom-field-' . $rnd_custom_field_id;
						}else{
							$gdlr_core_custom_field_id = empty($gdlr_core_custom_field_id)? 1: $gdlr_core_custom_field_id + 1;
							$settings['id'] = 'gdlr-core-custom-field-' . $gdlr_core_custom_field_id;
						}
					}

					$custom_style = str_replace('custom_style_id', $settings['id'], $custom_style); 
					if( $preview ){
						$custom_style = '<style>' . $custom_style . '</style>';
					}else{
						gdlr_core_add_inline_style($custom_style);
						$custom_style = '';
					}
				}

				// start printing item
				$extra_class  = 'gdlr-core-' . (empty($settings['text-align'])? 'left': $settings['text-align']) . '-align';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$ret  = '<div class="gdlr-core-custom-field-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				$ret .= gdlr_core_esc_style(array(
					'padding-bottom'=> (!empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb)? $settings['padding-bottom']: '',
					'margin-left'=>empty($settings['margin-left'])? '': $settings['margin-left'],
					'margin-right'=>empty($settings['margin-right'])? '': $settings['margin-right'],
				));
				
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				$content = '';

				if( $preview ){
					if( empty($settings['custom-field-slug']) ){
						$content = esc_html__('Please fill "Custom field slug"','goodlayers-core-cpt-template');
					}
					$content = sprintf(esc_html__('This option will display value from "%s" custom field.','goodlayers-core-cpt-template'), $settings['custom-field-slug']);
				}else if( !empty($settings['custom-field-slug']) ){
					$content = get_post_meta(get_the_ID(), $settings['custom-field-slug'], true);
				}

				if( !empty($content) ){
					$ret .= '<div class="gdlr-core-custom-field-item-content" ' . gdlr_core_esc_style(array(
						'font-size' => empty($settings['font-size'])? '': $settings['font-size'],
						'line-height' => empty($settings['content-line-height'])? '': $settings['content-line-height'],
						'font-weight' => empty($settings['content-font-weight'])? '': $settings['content-font-weight'],
						'letter-spacing' => empty($settings['content-letter-spacing'])? '': $settings['content-letter-spacing'],
						'text-transform' => empty($settings['content-text-transform'])? '': $settings['content-text-transform'],
						'color' => empty($settings['text-color'])? '': $settings['text-color']
					)) . ' >' . gdlr_core_content_filter($content) . '</div>';
				}
				$ret .= '</div>';
				$ret .= $custom_style;
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_text_box
	} // class_exists	