<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('icon', 'gdlr_core_pb_element_icon'); 
	
	if( !class_exists('gdlr_core_pb_element_icon') ){
		class gdlr_core_pb_element_icon{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'icon_lightbulb_alt',
					'title' => esc_html__('Icon', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'icon' => array(
								'title' => esc_html__('Icons', 'goodlayers-core'),
								'type' => 'icons',
								'default' => 'fa fa-gears',
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'icon-link' => array(
								'title' => esc_html__('Icon Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'icon-link-target' => array(
								'title' => esc_html__('Icon Link Target', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'_self' => esc_html__('Current Screen', 'goodlayers-core'),
									'_blank' => esc_html__('New Window', 'goodlayers-core'),
									'video-to-lightbox' => esc_html__('Video to Lightbox', 'goodlayers-core')
								)
							),			
						)
					),
					'style' => array(
						'title' => esc_html__('Icon Style', 'goodlayers-core'),
						'options' => array(
							'text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center'
							), 
							'icon-size' => array(
								'title' => esc_html__('Icon Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '45px',
							),
							'icon-color' => array(
								'title' => esc_html__('Icon Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'background' => array(
								'title' => esc_html__('Icon Background', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'round' => esc_html__('Round', 'goodlayers-core'),
									'circle' => esc_html__('Circle', 'goodlayers-core'),
								),
								'default' => 'none'
							), 
							'background-padding' => array(
								'title' => esc_html__('Background Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'20px', 'right'=>'20px', 'bottom'=>'20px', 'left'=>'20px', 'settings'=>'link' ),
								'condition' => array( 'background' => array('round', 'circle') )
							), 
							'border-radius' => array(
								'title' => esc_html__('Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0px',
								'data-input-type' => 'pixel',
								'condition' => array( 'background' => 'round' )
							),
							'background-color' => array(
								'title' => esc_html__('Icon Background Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'background' => array('round', 'circle') )
							),
							'background-border' => array(
								'title' => esc_html__('Icon Background Border', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'outer' => esc_html__('Outer', 'goodlayers-core'),
									'inner' => esc_html__('Inner', 'goodlayers-core'),
								),
								'default' => 'none',
								'condition' => array( 'background' => array('round', 'circle') )
							),
							'border-width' => array(
								'title' => esc_html__('Border Width', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'2px', 'right'=>'2px', 'bottom'=>'2px', 'left'=>'2px', 'settings'=>'link' ),
								'condition' => array( 'background' => array('round', 'circle'), 'background-border' => array('outer', 'inner') )
							),
							'border-color' => array(
								'title' => esc_html__('Icon Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'background' => array('round', 'circle'), 'background-border' => array('outer', 'inner') )
							),
							'pre-border-space' => array(
								'title' => esc_html__('Spaces Before Border', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'20px', 'right'=>'20px', 'bottom'=>'20px', 'left'=>'20px', 'settings'=>'link' ),
								'condition' => array( 'background' => array('round', 'circle'), 'background-border' =>'inner' )
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
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom ( Item )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
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
?><script id="gdlr-core-preview-content-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-content-<?php echo esc_attr($id); ?>').parent().gdlr_core_content_script();
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
						'text-align' => 'center', 'icon' => 'fa fa-gears', 'background' => 'none', 'background-border' => 'none',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// default value
				$settings['icon-size'] = (empty($settings['icon-size']) || $settings['icon-size']=='45px')? '': $settings['icon-size'];

				// start printing item
				$extra_class  = 'gdlr-core-' . (empty($settings['text-align'])? 'left': $settings['text-align']) . '-align';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$ret  = '<div class="gdlr-core-icon-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				$ret .= gdlr_core_esc_style(array(
					'padding-bottom'=> (!empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb)? $settings['padding-bottom']: '',
					'transform' => empty($settings['3d-content-z-pos'])? '': 'translateZ(' . $settings['3d-content-z-pos'] . ')'
				));
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				if( !empty($settings['icon-link']) ){
					if( !empty($settings['icon-link-target']) && $settings['icon-link-target'] == 'video-to-lightbox' ){
						$ret .= '<a ' . gdlr_core_get_lightbox_atts(array(
							'url' => $settings['icon-link']
						)) . ' >';
					}else{
						$ret .= '<a href="' . esc_attr($settings['icon-link']) . '" target="' . esc_attr((empty($settings['icon-link-target'])? '_self': $settings['icon-link-target'])) . '" >';
					}
				}

				if( !empty($settings['icon']) ){
					
					$icon_atts = array(
						'color' => empty($settings['icon-color'])? '': $settings['icon-color'],
						'font-size' => $settings['icon-size'],
						'min-width' => $settings['icon-size'],
						'min-height' => $settings['icon-size']
					);
					$icon_class  = ' gdlr-core-icon-item-icon';
					$icon_class .= ' ' . $settings['icon'];
					
					if( !empty($settings['background']) && $settings['background'] != 'none' ){
							
						$icon_class .= ' gdlr-core-skin-e-content';
							
						$wrapper_atts = array(
							'background-color' => empty($settings['background-color'])? '': $settings['background-color']
						);
						$wrapper_class  = ' gdlr-core-icon-item-wrap gdlr-core-skin-e-background';
						$wrapper_class .= ' gdlr-core-icon-item-type-' . $settings['background'];
						
						if( $settings['background'] == 'round' ){
							$wrapper_atts['border-radius'] = (empty($settings['border-radius']) || $settings['border-radius'] == '0px')? '': $settings['border-radius'];
						}
						
						if( !empty($settings['background-border']) && $settings['background-border'] == 'inner' ){
							$wrapper_class .= ' gdlr-core-with-border-inner';
							$wrapper_atts['padding'] = (empty($settings['pre-border-space']) || $settings['pre-border-space'] == array(
									array( 'top'=>'20px', 'right'=>'20px', 'bottom'=>'20px', 'left'=>'20px', 'settings'=>'link' )
								))? '': $settings['pre-border-space'];
							
							$icon_class .= ' gdlr-core-with-border gdlr-core-skin-border';
							$icon_atts['padding'] = (empty($settings['background-padding']) || $settings['background-padding'] == array(
									array( 'top'=>'20px', 'right'=>'20px', 'bottom'=>'20px', 'left'=>'20px', 'settings'=>'link' )
								))? '': $settings['background-padding'];	
							$icon_atts['border-color'] = empty($settings['border-color'])? '': $settings['border-color'];
							$icon_atts['border-width'] = (empty($settings['border-width']) || $settings['border-width'] == array( 
									'top'=>'2px', 'right'=>'2px', 'bottom'=>'2px', 'left'=>'2px', 'settings'=>'link'
								))? '': $settings['border-width'];
							if( $settings['background'] == 'round' ){
								$icon_atts['border-radius'] = (empty($settings['border-radius']) || $settings['border-radius'] == '0px')? '': $settings['border-radius'];
							}
						}else{
							$wrapper_atts['padding'] = (empty($settings['background-padding']) || $settings['background-padding'] == array(
									array( 'top'=>'20px', 'right'=>'20px', 'bottom'=>'20px', 'left'=>'20px', 'settings'=>'link' )
								))? '': $settings['background-padding'];
							
							if( !empty($settings['background-border']) && $settings['background-border'] == 'outer' ){
								$wrapper_class .= ' gdlr-core-with-border gdlr-core-skin-border';
								$wrapper_atts['border-color'] = empty($settings['border-color'])? '': $settings['border-color'];
								$wrapper_atts['border-width'] = (empty($settings['border-width']) || $settings['border-width'] == array( 
										'top'=>'2px', 'right'=>'2px', 'bottom'=>'2px', 'left'=>'2px', 'settings'=>'link'
									))? '': $settings['border-width'];
							}
							
						}
						
						$ret .= '<div class="' . esc_attr($wrapper_class) . '" ' . gdlr_core_esc_style($wrapper_atts) . ' >';
					}
					
					$ret .= '<i class="' . esc_attr($icon_class) . '" ' . gdlr_core_esc_style($icon_atts) . ' ></i>';
					
					if( !empty($settings['background']) && $settings['background'] != 'none' ){
						$ret .= '</div>'; // gdlr-core-icon-item-wrap
					}
				}
				
				if( !empty($settings['icon-link']) ){
					$ret .= '</a>';
				}
				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_icon
	} // class_exists	

	// [gdlr_core_icon icon="" size="" color="" margin-left="" margin-right="" ]
	add_shortcode('gdlr_core_icon', 'gdlr_core_icon_shortcode');
	if( !function_exists('gdlr_core_icon_shortcode') ){
		function gdlr_core_icon_shortcode($atts){
			$atts = shortcode_atts(array(
				'icon' => '',
				'size' => '',
				'color' => '',
				'margin-left' => '',
				'margin-right' => '',
				'link' => '',
				'link-target' => '_self',
				'padding' => '',
				'background-color' => '',
				'border-radius' => ''
			), $atts, 'gdlr_core_icon');
			
			$ret  = '<i class="' . esc_attr($atts['icon']) . '" ' . gdlr_core_esc_style(array(
				'font-size' => $atts['size'],
				'color' => $atts['color'],
				'margin-left' => $atts['margin-left'],
				'margin-right' => $atts['margin-right'],
				'padding' => $atts['padding'],
				'background-color' => $atts['background-color'],
				'border-radius' => $atts['border-radius']
			)) . ' ></i>';

			if( !empty($atts['link']) ){
				$ret = '<a href="' . esc_url($atts['link']) . '" target="' . esc_attr($atts['link-target']) . '" >' . $ret . '</a>';
			}

			return $ret;
		}
	}	