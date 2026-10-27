<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('divider', 'gdlr_core_pb_element_divider'); 
	
	if( !class_exists('gdlr_core_pb_element_divider') ){
		class gdlr_core_pb_element_divider{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-minus',
					'title' => esc_html__('Divider', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(	
							'type' => array(
								'title' => esc_html__('Divider Type', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'normal' => GDLR_CORE_URL . '/include/images/divider/normal.png',
									'center-circle' => GDLR_CORE_URL . '/include/images/divider/center-circle.png',
									'with-icon' => GDLR_CORE_URL . '/include/images/divider/with-icon.png',
									'small-center' => GDLR_CORE_URL . '/include/images/divider/small-center.png',
									'small-left' => GDLR_CORE_URL . '/include/images/divider/small-left.png',
									'small-right' => GDLR_CORE_URL . '/include/images/divider/small-right.png',
									'dot' => GDLR_CORE_URL . '/include/images/divider/dot.jpg',
								),
								'default' => 'normal',
								'wrapper-class' => 'gdlr-core-fullsize'
							), 
							'icon-type' => array(
								'title' => esc_html__('Icon Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'icon' => esc_html__('Icon', 'goodlayers-core'),
									'image' => esc_html__('Image', 'goodlayers-core'),
								),
								'condition' => array( 'type' => 'with-icon' ),
							), 
							'image' => array(
								'title' => esc_html__('Upload Icon', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'type' => 'with-icon', 'icon-type' => 'image' ),
							),
							'icon' => array(
								'title' => esc_html__('Icon', 'goodlayers-core'),
								'type' => 'icons',
								'default' => 'fa fa-film',
								'wrapper-class' => 'gdlr-core-fullsize',
								'condition' => array( 'type' => 'with-icon', 'icon-type' => 'icon' ),
								
							),
							'style' => array(
								'title' => esc_html__('Divider Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'solid' => GDLR_CORE_URL . '/include/images/divider/solid.png',
									'double' => GDLR_CORE_URL . '/include/images/divider/double.png',
									'dotted' => GDLR_CORE_URL . '/include/images/divider/dotted.png',
									'dashed' => GDLR_CORE_URL . '/include/images/divider/dashed.png'
								),
								'default' => 'solid',
								'condition' => array( 'type' => array('normal', 'with-icon') ),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'align' => array(
								'title' => esc_html__('Divider Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center',
								'condition' => array( 'type' => array('normal', 'with-icon', 'dot') )
							),
							'divider-type' => array(
								'title' => esc_html__('Divider Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'horizontal' => esc_html__('Horizontal', 'goodlayers-core'),
									'vertical' => esc_html__('Vertical', 'goodlayers-core'),
								),
								'default' => 'horizontal',
								'condition' => array( 'type' => array('normal') )
							),
							'vertical-divider-icon' => array(
								'title' => esc_html__('Vertical Divider Icon', 'goodlayers-core'),
								'type' => 'icons',
								'allow-none' => true,
								'wrapper-class' => 'gdlr-core-fullsize',
								'condition' => array( 'type' => array('normal'), 'divider-type' => 'vertical' )
							),
							'vertical-divider-text' => array(
								'title' => esc_html__('Vertical Divider Text', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'type' => array('normal'), 'divider-type' => 'vertical' )
							),
							'icon-size' => array(
								'title' => esc_html__('Icon Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '15px',
								'condition' => array( 'type' => 'with-icon', 'icon-type' => 'icon' ),
							),
							'divider-size' => array(
								'title' => esc_html__('Divider Height', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '1px',
								'condition' => array( 'type' => array('normal', 'with-icon', 'dot') ),
								'description' => esc_html__('At least "4px" is required for double divider style', 'goodlayers-core')
							),
							'divider-width' => array(
								'title' => esc_html__('Divider Width', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '',
								'condition' => array( 'type' => array('normal', 'with-icon', 'small-center', 'small-left', 'small-right') )
							),
							'divider-border-radius' => array(
								'title' => esc_html__('Divider Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '',
								'condition' => array( 'type' => array('normal', 'with-icon', 'small-center', 'small-left', 'small-right') )
							), 
							'divider-skewx' => array(
								'title' => esc_html__('Divider Skew X', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'condition' => array( 'type' => array('normal', 'with-icon') ),
								'description' => esc_html__('Only input number here', 'goodlayers-core')
							)				
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'margin-top' => array(
								'title' => esc_html__('Margin Top', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'margin-left' => array(
								'title' => esc_html__('Margin Left', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'margin-right' => array(
								'title' => esc_html__('Margin Right', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'padding-bottom' => array(
								'title' => esc_html__('Margin Bottom', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),
						)
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'vdi-font-size' => array(
								'title' => esc_html__('Vertical Divider Icon Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'vd-font-size' => array(
								'title' => esc_html__('Vertical Divider Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'vd-font-weight' => array(
								'title' => esc_html__('Vertical Divider Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'default' => '',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'vd-font-letter-spacing' => array(
								'title' => esc_html__('Vertical Divider Font Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'vd-text-transform' => array(
								'title' => esc_html__('Title Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'data-type' => 'text',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => 'uppercase'
							),
						)
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'icon-color' => array(
								'title' => esc_html__('Icon Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'divider-color' => array(
								'title' => esc_html__('Divider Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
						)
					),
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-divider-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-divider-<?php echo esc_attr($id); ?>').parent().gdlr_core_divider();
});
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array() ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'type' => 'normal', 'icon-type' => 'icon', 'image' => '', 'icon' => 'fa fa-film', 'icon-size' => '15px',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				$skewx = empty($settings['divider-skewx'])? '': intval($settings['divider-skewx']);

				$border_atts = array(
					'border-color' => (empty($settings['divider-color'])? '': $settings['divider-color']),
					'border-bottom-style' => ((empty($settings['style']) || $settings['style'] == 'solid')? '': $settings['style']),
					'border-width' => ((empty($settings['divider-size']) || $settings['divider-size'] == '1px')? '': $settings['divider-size']),
					'border-radius' => empty($settings['divider-border-radius'])? '': $settings['divider-border-radius'],
					'skewx' => $skewx
				);

				$settings['type'] = empty($settings['type'])? 'normal': $settings['type'];
				$settings['align'] = empty($settings['align'])? 'center': $settings['align'];
				$settings['divider-type'] = empty($settings['divider-type'])? 'horizontal': $settings['divider-type'];
					
				// start printing item
				$extra_class  = 'gdlr-core-divider-item-' . $settings['type'];
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$extra_class .= empty($settings['no-pdlr'])? ' gdlr-core-item-pdlr': '';
				if( $settings['type'] == 'normal' || $settings['type'] == 'with-icon' ){
					$extra_class .= ' gdlr-core-' . (empty($settings['align'])? 'center': $settings['align']) . '-align';
				}
				if( $settings['type'] == 'normal' ){
					if( $settings['divider-type'] == 'vertical' ){
						$extra_class .= ' gdlr-core-style-vertical';
						if( !empty($settings['divider-width']) ){
							$border_atts['height'] = $settings['divider-width'];
							$settings['divider-width'] = '';
						}
					}
				}

				$ret  = '<div class="gdlr-core-divider-item ' . esc_attr($extra_class) . '" ';
				$ret .= gdlr_core_esc_style(array(
					'margin-top' => (empty($settings['margin-top']) || $settings['margin-top'] == $gdlr_core_item_pdb)? '': $settings['margin-top'],
					'margin-bottom' => (empty($settings['padding-bottom']) || $settings['padding-bottom'] == $gdlr_core_item_pdb)? '': $settings['padding-bottom'],
					'margin-left' => empty($settings['margin-left'])? '': $settings['margin-left'],
					'margin-right' => empty($settings['margin-right'])? '': $settings['margin-right'],
				));
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( !empty($settings['divider-width']) ){
					$ret .= '<div class="gdlr-core-divider-container" ' . gdlr_core_esc_style(array(
						'max-width' => $settings['divider-width']
					)) . ' >';
				}

				if( $settings['type'] == 'normal' ){
					$ret .= '<div class="gdlr-core-divider-line gdlr-core-skin-divider" ' . gdlr_core_esc_style($border_atts) . '></div>';
					
					if( $settings['divider-type'] == 'vertical' && 
						(!empty($settings['vertical-divider-text']) || !empty($settings['vertical-divider-icon'])) ){
						$ret .= '<div class="gdlr-core-divider-line-vertical-text" ' . gdlr_core_esc_style(array(
							'color' => (empty($settings['divider-color'])? '': $settings['divider-color']),
							'font-size' => empty($settings['vd-font-size'])? '': $settings['vd-font-size'],
							'font-weight' => empty($settings['vd-font-weight'])? '': $settings['vd-font-weight'],
							'letter-spacing' => empty($settings['vd-font-letter-spacing'])? '': $settings['vd-font-letter-spacing'],
							'text-transform' => empty($settings['vd-text-transform'])? '': $settings['vd-text-transform'],
						)) . ' >';
						if( !empty($settings['vertical-divider-icon']) ){
							$ret .= '<i class="gdlr-core-divider-line-vertical-icon ' . esc_attr($settings['vertical-divider-icon']) . '" ' . gdlr_core_esc_style(array(
								'color' => (empty($settings['divider-color'])? '': $settings['divider-color']),
								'font-size' => empty($settings['vdi-font-size'])? '': $settings['vdi-font-size'],
							)) . ' ></i>';
						}
						$ret .= gdlr_core_text_filter($settings['vertical-divider-text']);
						$ret .= '</div>';
					}
				}else if( $settings['type'] == 'dot' ){
					$ret .= '<div class="gdlr-core-divider-container" ' . gdlr_core_esc_style(array(
						'width' => $settings['divider-size'],
						'height' => $settings['divider-size'],
						'background' => $settings['divider-color']
					)) . ' ></div>';

				}else if( $settings['type'] == 'with-icon' ){
					$ret .= '<div class="gdlr-core-divider-item-with-icon-inner gdlr-core-js">';
					$ret .= '<div class="gdlr-core-divider-line gdlr-core-left gdlr-core-skin-divider" ' . gdlr_core_esc_style($border_atts) . '></div>';
					if( $settings['icon-type'] == 'icon' ){
						if( !empty($settings['icon']) ){
							$ret .= '<i class="' . esc_attr($settings['icon']) . '" ' . gdlr_core_esc_style(array(
								'color' => (empty($settings['icon-color'])? '': $settings['icon-color']),
								'font-size' => ((empty($settings['icon-size']) || $settings['icon-size'] == '15px')? '': $settings['icon-size'])
							)) . ' ></i>';
						}
					}else if( $settings['icon-type'] == 'image' ){
						if( !empty($settings['image']) ){
							$ret .= gdlr_core_get_image($settings['image']);
						}
					}
					$ret .= '<div class="gdlr-core-divider-line gdlr-core-skin-divider gdlr-core-right" ' . gdlr_core_esc_style($border_atts) . '></div>';
					$ret .= '</div>';
				}else{
					$divider_color = array( 
						'border-color' => (empty($settings['divider-color'])? '': $settings['divider-color']) 
					);
					if( $settings['type'] == 'center-circle' ){
						$divider_color['color'] = $divider_color['border-color'];
					}

					$ret .= '<div class="gdlr-core-divider-line gdlr-core-skin-divider" ' . gdlr_core_esc_style($divider_color) . ' >';
					$ret .= '<div class="gdlr-core-divider-line-bold  gdlr-core-skin-divider" ' . gdlr_core_esc_style($divider_color) . ' ></div>';
					$ret .= '</div>';
				}

				if( !empty($settings['divider-width']) ){
					$ret .= '</div>'; // gdlr-core-divider-container
				}
				
				$ret .= '</div>'; // gdlr-core-divider-item
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_divider
	} // class_exists	

	// [gdlr_core_divider divider-size="3px" divider-color="#f2ff3d" divider-width="150px" align="left" ]
	add_shortcode('gdlr_core_divider', 'gdlr_core_divider_shortcode');
	if( !function_exists('gdlr_core_divider_shortcode') ){
		function gdlr_core_divider_shortcode($atts, $content = ''){
			$atts = wp_parse_args($atts, array(
				'no-pdlr' => true,
				'type' => 'normal',
				'style' => 'solid',
				'align' => 'center',
			));

			return gdlr_core_pb_element_divider::get_content($atts);
		}
	}