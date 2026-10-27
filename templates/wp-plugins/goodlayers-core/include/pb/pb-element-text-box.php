<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('text-box', 'gdlr_core_pb_element_text_box'); 
	
	if( !class_exists('gdlr_core_pb_element_text_box') ){
		class gdlr_core_pb_element_text_box{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-align-justify',
					'title' => esc_html__('Text Box', 'goodlayers-core')
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
								'type' => 'tinymce',
								'default' => esc_html__('Text box item sample content', 'goodlayers-core'),
								'wrapper-class' => 'gdlr-core-fullsize'
							),		
							'text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'left'
							),		
							'apply-the-content-filter' => array(
								'title' => esc_html__('Apply The Content Filter', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'description' => esc_html__('You may try enable this option when shortcode is not working with some plugin.', 'goodlayers-core')
							),
							'enable-p-space' => array(
								'title' => esc_html__('Enable Paragraph Spaces', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable'
							)
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
							'enable-text-frame' => array(
								'title' => esc_html__('Enable Text Frame', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'frame-padding' => array(
								'title' => esc_html__('Frame Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'unlink' ),
								'condition' => array( 'enable-text-frame' => 'enable' )
							),
							'frame-background' => array(
								'title' => esc_html__('Frame Background', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'frame-border-radius' => array(
								'title' => esc_html__('Frame Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'frame-shadow-size' => array(
								'title' => esc_html__('Shadow Size ( for image/frame )', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
								'condition' => array( 'enable-text-frame' => 'enable' )
							),
							'frame-shadow-color' => array(
								'title' => esc_html__('Shadow Color ( for image/frame )', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'enable-text-frame' => 'enable' )
							),
							'frame-shadow-opacity' => array(
								'title' => esc_html__('Shadow Opacity ( for image/frame )', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core'),
								'condition' => array( 'enable-text-frame' => 'enable' )
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'3d-content-z-pos' => array(
								'title' => esc_html__('3D Z Position ( For 3D Column Effect )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
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
						'content' => esc_html__('Text box item sample content', 'goodlayers-core'),
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
						global $gdlr_core_text_box_id;

						if( $preview ){
							$gdlr_core_text_box_id = empty($gdlr_core_text_box_id)? array(): $gdlr_core_text_box_id;

							// generate unique id so it does not get overwritten in admin area
							$rnd_text_box_id = mt_rand(0, 99999);
							while( in_array($rnd_text_box_id, $gdlr_core_text_box_id) ){
								$rnd_text_box_id = mt_rand(0, 99999);
							}
							$gdlr_core_text_box_id[] = $rnd_text_box_id;
							$settings['id'] = 'gdlr-core-text-box-' . $rnd_text_box_id;
						}else{
							$gdlr_core_text_box_id = empty($gdlr_core_text_box_id)? 1: $gdlr_core_text_box_id + 1;
							$settings['id'] = 'gdlr-core-text-box-' . $gdlr_core_text_box_id;
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
				$extra_class .= (!empty($settings['enable-p-space']) && $settings['enable-p-space'] == 'disable')? ' gdlr-core-no-p-space': '';
				$ret  = '<div class="gdlr-core-text-box-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				$ret .= gdlr_core_esc_style(array(
					'padding-bottom'=> (!empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb)? $settings['padding-bottom']: '',
					'margin-left'=>empty($settings['margin-left'])? '': $settings['margin-left'],
					'margin-right'=>empty($settings['margin-right'])? '': $settings['margin-right'],
					'transform' => empty($settings['3d-content-z-pos'])? '': 'translateZ(' . $settings['3d-content-z-pos'] . ')'
				));
				
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				if( !empty($settings['content']) ){
					$the_content_filter = (empty($settings['apply-the-content-filter']) || $settings['apply-the-content-filter'] == 'disable')? true: false;
					$ret .= '<div class="gdlr-core-text-box-item-content" ' . gdlr_core_esc_style(array(
						'font-size' => empty($settings['font-size'])? '': $settings['font-size'],
						'line-height' => empty($settings['content-line-height'])? '': $settings['content-line-height'],
						'font-weight' => empty($settings['content-font-weight'])? '': $settings['content-font-weight'],
						'letter-spacing' => empty($settings['content-letter-spacing'])? '': $settings['content-letter-spacing'],
						'text-transform' => empty($settings['content-text-transform'])? '': $settings['content-text-transform'],
						'color' => empty($settings['text-color'])? '': $settings['text-color']
					)) . ' >';
					if( !empty($settings['enable-text-frame']) && $settings['enable-text-frame'] == 'enable' ){
						$ret .= '<div class="gdlr-core-text-box-frame" ' . gdlr_core_esc_style(array(
							'padding' => empty($settings['frame-padding'])? '': $settings['frame-padding'],
							'background-color' => empty($settings['frame-background'])? '': $settings['frame-background'],
							'border-radius' => empty($settings['frame-border-radius'])? '': $settings['frame-border-radius'],
							'background-shadow-size' => empty($settings['frame-shadow-size'])? '': $settings['frame-shadow-size'],
							'background-shadow-color' => empty($settings['frame-shadow-color'])? '': $settings['frame-shadow-color'],
							'background-shadow-opacity' => empty($settings['frame-shadow-opacity'])? '': $settings['frame-shadow-opacity'],
						)) . ' >';
					}
					$ret .= gdlr_core_content_filter($settings['content']);
					if( !empty($settings['enable-text-frame']) && $settings['enable-text-frame'] == 'enable' ){
						$ret .= '</div>';
					}
					$ret .= '</div>';
				}
				$ret .= '</div>';
				$ret .= $custom_style;
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_text_box
	} // class_exists	