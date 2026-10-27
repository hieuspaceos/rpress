<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('column-service', 'gdlr_core_pb_element_column_service'); 
	
	if( !class_exists('gdlr_core_pb_element_column_service') ){
		class gdlr_core_pb_element_column_service{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-align-left',
					'title' => esc_html__('Column Service', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;

				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'media-type' => array(
								'title' => esc_html__('Media Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'icon' => esc_html__('Icon', 'goodlayers-core'),
									'image' => esc_html__('Image', 'goodlayers-core'),
									'character' => esc_html__('Character', 'goodlayers-core'),
								),
								'default' => 'icon',
							),
							'icon' => array(
								'title' => esc_html__('Icon Selector', 'goodlayers-core'),
								'type' => 'icons',
								'default' => 'fa fa-gear',
								'wrapper-class' => 'gdlr-core-fullsize',
								'condition' => array( 'media-type' => 'icon' )
							),
							'icon-style' => array(
								'title' => esc_html__('Icon Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'normal' => esc_html__('Normal', 'goodlayers-core'),
									'round' => esc_html__('Round Background', 'goodlayers-core'),
								),
								'condition' => array( 'media-type' => 'icon' )
							),
							'image' => array(
								'title' => esc_html__('Upload', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'media-type' => 'image' )
							),
							'character' => array(
								'title' => esc_html__('Character', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'media-type' => 'character' )
							),
							'title' => array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Column Service Title', 'goodlayers-core'),
							),
							'caption' => array(
								'title' => esc_html__('Caption', 'goodlayers-core'),
								'type' => 'textarea',
							),
							'caption-position' => array(
								'title' => esc_html__('Caption Position', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'bottom' => esc_html__('Bottom', 'goodlayers-core'),
									'top' => esc_html__('Top', 'goodlayers-core'),
								)
							),
							'content' => array(
								'title' => esc_html__('Content', 'goodlayers-core'),
								'type' => 'tinymce',
								'default' => esc_html__('Column service content area', 'goodlayers-core'),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'read-more-text' => array(
								'title' => esc_html__('Read More Text', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Read More', 'goodlayers-core')
							),
							'read-more-link' => array(
								'title' => esc_html__('Read More Link', 'goodlayers-core'),
								'type' => 'text',
								'default' => '#'
							),
							'read-more-link-target' => array(
								'title' => esc_html__('Read More Link Target', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'_self' => esc_html__('Current Screen', 'goodlayers-core'),
									'_blank' => esc_html__('New Window', 'goodlayers-core'),
								),
								'default' => '_self'
							),
							'enable-read-more-icon' => array(
								'title' => esc_html__('Read More Icon', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'read-more-icon' => array(
								'title' => esc_html__('Read More Icon', 'goodlayers-core'),
								'type' => 'icons',
								'default' => '',
								'wrapper-class' => 'gdlr-core-fullsize',
								'condition' => array( 'enable-read-more-icon' => 'enable' )
							),
						),
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'style' => array(
								'title' => esc_html__('Column Service Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'center_icon-top' => GDLR_CORE_URL . '/include/images/column-service/center-icon-top.png',
									'left_icon-top' => GDLR_CORE_URL . '/include/images/column-service/left-icon-top.png',
									'left_icon-left' => GDLR_CORE_URL . '/include/images/column-service/left-icon-left.png',
									'left_icon-left-title' => GDLR_CORE_URL . '/include/images/column-service/left-icon-left-title.png',
									'right_icon-top' => GDLR_CORE_URL . '/include/images/column-service/right-icon-top.png',
									'right_icon-left' => GDLR_CORE_URL . '/include/images/column-service/right-icon-left.png',
									'right_icon-left-title' => GDLR_CORE_URL . '/include/images/column-service/right-icon-left-title.png',
								),
								'default' => 'center_icon-top',
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'heading-tag' => array(
								'title' => esc_html__('Heading Tag', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'h1' => esc_html__('H1', 'goodlayers-core'),
									'h2' => esc_html__('H2', 'goodlayers-core'),
									'h3' => esc_html__('H3', 'goodlayers-core'),
									'h4' => esc_html__('H4', 'goodlayers-core'),
									'h5' => esc_html__('H5', 'goodlayers-core'),
									'h6' => esc_html__('H6', 'goodlayers-core'),
									'div' => esc_html__('Div', 'goodlayers-core'),
								),
								'default' => 'h3'
							),
						),
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'icon-size' => array(
								'title' => esc_html__('Icon/Character Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '30px'
							),
							'title-size' => array(
								'title' => esc_html__('Title Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '14px'
							),
							'tablet-title-size' => array(
								'title' => esc_html__('Tablet Title Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'mobile-title-size' => array(
								'title' => esc_html__('Mobile Title Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'title-font-weight' => array(
								'title' => esc_html__('Title Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'title-font-style' => array(
								'title' => esc_html__('Title Font Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'default' => esc_html__('Default', 'goodlayers-core'),
									'normal' => esc_html__('Normal', 'goodlayers-core'),
									'italic' => esc_html__('Italic', 'goodlayers-core'),
								),
							),
							'title-letter-spacing' => array(
								'title' => esc_html__('Title Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'title-text-transform' => array(
								'title' => esc_html__('Title Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
									'none' => esc_html__('None', 'goodlayers-core'),
								),
								'default' => 'uppercase'
							),
							'caption-size' => array(
								'title' => esc_html__('Caption Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '14px'
							),
							'tablet-caption-size' => array(
								'title' => esc_html__('Tablet Caption Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'mobile-caption-size' => array(
								'title' => esc_html__('Mobile Caption Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'caption-font-weight' => array(
								'title' => esc_html__('Caption Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'caption-font-style' => array(
								'title' => esc_html__('Caption Font Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'default' => esc_html__('Default', 'goodlayers-core'),
									'normal' => esc_html__('Normal', 'goodlayers-core'),
									'italic' => esc_html__('Italic', 'goodlayers-core'),
								),
							),
							'caption-text-transform' => array(
								'title' => esc_html__('Caption Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => 'uppercase'
							),
							'content-size' => array(
								'title' => esc_html__('Content Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '15px'
							),
							'tablet-content-size' => array(
								'title' => esc_html__('Tablet Content Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'mobile-content-size' => array(
								'title' => esc_html__('Mobile Content Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'content-font-weight' => array(
								'title' => esc_html__('Content Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
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
							'read-more-size' => array(
								'title' => esc_html__('Read More Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '14px'
							),
							'read-more-font-weight' => array(
								'title' => esc_html__('Read More Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'read-more-font-style' => array(
								'title' => esc_html__('Read More Font Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'default' => esc_html__('Default', 'goodlayers-core'),
									'normal' => esc_html__('Normal', 'goodlayers-core'),
									'italic' => esc_html__('Italic', 'goodlayers-core'),
								),
							),
							'read-more-icon-font-size' => array(
								'title' => esc_html__('Read More Icon Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
						)
					),					
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'icon-color' => array(
								'title' => esc_html__('Icon/Character Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'icon-background' => array(
								'title' => esc_html__('Icon Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'background-color' => array(
								'title' => esc_html__('Column Service Background Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
						)
					),
					'background' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(						
							'media-margin' => array(
								'title' => esc_html__('Icon/Image Margin', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'wrapper-class' => 'gdlr-core-no-link',
								'description' => esc_html__('Leave this field blank for default value.', 'goodlayers-core'),
							),						
							'media-image-max-width' => array(
								'title' => esc_html__('Media Image Max Width', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),					
							'character-width' => array(
								'title' => esc_html__('Character Width', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'title-top-padding' => array(
								'title' => esc_html__('Title Top Padding', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'description' => esc_html__('Leave this field blank for default value.', 'goodlayers-core'),
							),
							'title-bottom-margin' => array(
								'title' => esc_html__('Title Bottom Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),							
							'caption-top-margin' => array(
								'title' => esc_html__('Caption Top Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'read-more-top-margin' => array(
								'title' => esc_html__('Read More Top Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'description' => esc_html__('The value will be added to the initial value ( 20px )', 'goodlayers-core'),
							),
							'padding' => array(
								'title' => esc_html__('Padding Spaces ( Inside Background )', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'wrapper-class' => 'gdlr-core-no-link',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>$gdlr_core_item_pdb, 'left'=>'', 'settings'=>'unlink' )
							),
							'margin' => array(
								'title' => esc_html__('Padding Spaces ( Outside Background )', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link' )
							),
						)
					)
				);
			}

			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-column-service-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-column-service-<?php echo esc_attr($id); ?>').parent().gdlr_core_content_script();
});
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}		
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){

				// default variable
				if( empty($settings) ){
					$settings = array(
						'icon' => 'fa fa-gear',
						'title' => esc_html__('Column Service Title', 'goodlayers-core'),
						'caption' => '',
						'content' => esc_html__('Column service content area', 'goodlayers-core'),
						'read-more-text' => esc_html__('Read More', 'goodlayers-core'),
						'read-more-link' => '#',
						'style' => 'center_icon-top'
					);
				}

				$custom_style = '';
				if( !empty($settings['tablet-title-size']) ){
					$custom_style .= '@media only screen and (max-width: 999px){';
					$custom_style .= '#custom_style_id .gdlr-core-column-service-title{ ' . gdlr_core_esc_style(array(
						'font-size' => $settings['tablet-title-size']
					), false, true) . ' }';
					$custom_style .= '}';
				}
				if( !empty($settings['mobile-title-size']) ){
					$custom_style .= '@media only screen and (max-width: 767px){';
					$custom_style .= '#custom_style_id .gdlr-core-column-service-title{ ' . gdlr_core_esc_style(array(
						'font-size' => $settings['mobile-title-size']
					), false, true) . ' }';
					$custom_style .= '}';
				}
				if( !empty($settings['tablet-caption-size']) ){
					$custom_style .= '@media only screen and (max-width: 999px){';
					$custom_style .= '#custom_style_id .gdlr-core-column-service-caption{ ' . gdlr_core_esc_style(array(
						'font-size' => $settings['tablet-caption-size']
					), false, true) . ' }';
					$custom_style .= '}';
				}
				if( !empty($settings['mobile-caption-size']) ){
					$custom_style .= '@media only screen and (max-width: 767px){';
					$custom_style .= '#custom_style_id .gdlr-core-column-service-caption{ ' . gdlr_core_esc_style(array(
						'font-size' => $settings['mobile-caption-size']
					), false, true) . ' }';
					$custom_style .= '}';
				}
				if( !empty($settings['tablet-content-size']) ){
					$custom_style .= '@media only screen and (max-width: 999px){';
					$custom_style .= '#custom_style_id .gdlr-core-column-service-content{ ' . gdlr_core_esc_style(array(
						'font-size' => $settings['tablet-content-size']
					), false, true) . ' }';
					$custom_style .= '}';
				}
				if( !empty($settings['mobile-content-size']) ){
					$custom_style .= '@media only screen and (max-width: 767px){';
					$custom_style .= '#custom_style_id .gdlr-core-column-service-content{ ' . gdlr_core_esc_style(array(
						'font-size' => $settings['mobile-content-size']
					), false, true) . ' }';
					$custom_style .= '}';
				}
				if( !empty($custom_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_column_service_id;
						if( $preview ){
							$gdlr_core_column_service_id = empty($gdlr_core_column_service_id)? array(): $gdlr_core_column_service_id;

							// generate unique id so it does not get overwritten in admin area
							$rnd_column_service_id = mt_rand(0, 99999);
							while( in_array($rnd_column_service_id, $gdlr_core_column_service_id) ){
								$rnd_column_service_id = mt_rand(0, 99999);
							}
							$gdlr_core_column_service_id[] = $rnd_column_service_id;
							$settings['id'] = 'gdlr-core-column-service-' . $rnd_column_service_id;
						}else{
							$gdlr_core_column_service_id = empty($gdlr_core_column_service_id)? 1: $gdlr_core_column_service_id + 1;
							$settings['id'] = 'gdlr-core-column-service-' . $gdlr_core_column_service_id;
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
				
				// default size
				$settings['style'] = empty($settings['style'])? '': $settings['style'];
				$settings['icon-size'] = (empty($settings['icon-size']) || $settings['icon-size'] == '30px')? '': $settings['icon-size'];
				$settings['title-size'] = (empty($settings['title-size']) || $settings['title-size'] == '14px')? '': $settings['title-size'];
				$settings['caption-size'] = (empty($settings['caption-size']) || $settings['caption-size'] == '14px')? '': $settings['caption-size'];
				$settings['content-size'] = (empty($settings['content-size']) || $settings['content-size'] == '15px')? '': $settings['content-size'];
				$settings['read-more-size'] = (empty($settings['read-more-size']) || $settings['read-more-size'] == '14px')? '': $settings['read-more-size'];
				
				$cs_style = explode('_', $settings['style']);
				$text_align = empty($cs_style[0])? 'center': $cs_style[0];
				$style = empty($cs_style[1])? 'icon-top': $cs_style[1];
				
				// start printing item
				$extra_class  = ' gdlr-core-' . $text_align . '-align';
				$extra_class .= ($text_align == 'center')? '': ' gdlr-core-column-service-' . $style;
				$extra_class .= empty($settings['caption'])? ' gdlr-core-no-caption': ' gdlr-core-with-caption';
				$extra_class .= empty($settings['no-pdlr'])? ' gdlr-core-item-pdlr': '';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$ret  = '<div class="gdlr-core-column-service-item gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				$ret .= gdlr_core_esc_style(array(
					'background-color' => empty($settings['background-color'])? '': $settings['background-color'],
					'padding'=> (empty($settings['padding']) || $settings['padding'] == array(
						'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'unlink'
					))? '': $settings['padding'],
					'margin'=> (empty($settings['margin']) || $settings['margin'] == array(
						'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link'
					))? '': $settings['margin'],
				));
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				
				// media
				$media_html = '';
				if( empty($settings['media-type']) || $settings['media-type'] != 'none' ){
					$media_atts = array(
						'margin' => empty($settings['media-margin'])? array(): $settings['media-margin'],
					); 
				
					if( empty($settings['media-type']) || $settings['media-type'] == 'icon' ){
						$icon_class = '';
						$icon_inner_class = '';
						if( !empty($settings['icon-style']) && $settings['icon-style'] == 'round' ){
							$icon_class .= ' gdlr-core-icon-style-round';
							$icon_inner_class .= ' gdlr-core-skin-e-background gdlr-core-skin-e-content';
						}

						$media_html  = '<div class="gdlr-core-column-service-media gdlr-core-media-icon ' . $icon_class . '" ' . gdlr_core_esc_style($media_atts) . ' >';
						$media_html .= '<i class="' . esc_attr($settings['icon']) . esc_attr($icon_inner_class) . '" ' . gdlr_core_esc_style(array(
							'font-size'=>$settings['icon-size'],
							'line-height'=>$settings['icon-size'],
							'width'=>$settings['icon-size'],
							'color'=>(empty($settings['icon-color'])? '': $settings['icon-color']),
							'background-color'=>(empty($settings['icon-background'])? '': $settings['icon-background'])
						)) . ' ></i>';
						$media_html .= '</div>';
					}else if( !empty($settings['media-type']) && $settings['media-type'] == 'image' && !empty($settings['image']) ){
						if( !empty($settings['media-image-max-width']) ){
							$media_atts['max-width'] = $settings['media-image-max-width'];
						}
						if( $text_align == 'center' ){
							$media_atts['margin-left'] = 'auto';
							$media_atts['margin-right'] = 'auto';
						}
						$media_html  = '<div class="gdlr-core-column-service-media gdlr-core-media-image" ' . gdlr_core_esc_style($media_atts) . ' >';
						$media_html .= gdlr_core_get_image($settings['image']);
						$media_html .= '</div>';
					}else if( !empty($settings['media-type']) && $settings['media-type'] == 'character' && !empty($settings['character']) ){
						$media_atts['font-size'] = $settings['icon-size'];
						$media_atts['color'] = (empty($settings['icon-color'])? '': $settings['icon-color']);
						$media_atts['width'] = (empty($settings['character-width'])? '': $settings['character-width']);

						$media_html  = '<div class="gdlr-core-column-service-media gdlr-core-character" ' . gdlr_core_esc_style($media_atts) . ' >';
						$media_html .= gdlr_core_text_filter($settings['character']);
						$media_html .= '</div>';
					}
				}
				
				if( ($text_align == 'center' || $style != 'icon-left-title') ){
					$ret .= $media_html;
					$ret .= '<div class="gdlr-core-column-service-content-wrapper" >';
				}else{
					$ret .= '<div class="gdlr-core-column-service-content-wrapper" >';
					$ret .= $media_html;
				}
				
				// title
				if( !empty($settings['title']) || !empty($settings['caption']) ){
					if( !empty($settings['caption']) ){
						$caption = '<div class="gdlr-core-column-service-caption gdlr-core-info-font gdlr-core-skin-caption" ' . gdlr_core_esc_style(array(
							'font-size' => $settings['caption-size'],
							'font-weight' => empty($settings['caption-font-weight'])? '': $settings['caption-font-weight'],
							'font-style' => (empty($settings['caption-font-style']) || $settings['caption-font-style'] == 'default')? '': $settings['caption-font-style'],
							'text-transform' => empty($settings['caption-text-transform'])? '': $settings['caption-text-transform'],
							'margin-top' => empty($settings['caption-top-margin'])? '': $settings['caption-top-margin']
						)) . ' >' . gdlr_core_text_filter($settings['caption']) . '</div>';
					}

					$ret .= '<div class="gdlr-core-column-service-title-wrap" ' . gdlr_core_esc_style(array(
						'margin-bottom' => empty($settings['title-bottom-margin'])? '': $settings['title-bottom-margin']
					)) . ' >';
					if( !empty($caption) && (!empty($settings['caption-position']) && $settings['caption-position'] == 'top') ){
						$ret .= $caption;
					}
					if( !empty($settings['title']) ){
						$heading_tag = empty($settings['heading-tag'])? 'h3': $settings['heading-tag'];

						$ret .= '<' . $heading_tag . ' class="gdlr-core-column-service-title gdlr-core-skin-title" ' . gdlr_core_esc_style(array(
							'font-size' => $settings['title-size'],
							'padding-top' => empty($settings['title-top-padding'])? '': $settings['title-top-padding'],
							'font-weight' => empty($settings['title-font-weight'])? '': $settings['title-font-weight'],
							'font-style' => (empty($settings['title-font-style']) || $settings['title-font-style'] == 'default')? '': $settings['title-font-style'],
							'letter-spacing' => empty($settings['title-letter-spacing'])? '': $settings['title-letter-spacing'],
							'text-transform' => (empty($settings['title-text-transform']) || $settings['title-text-transform'] == 'uppercase')? '': $settings['title-text-transform']
						)) . ' >' . gdlr_core_text_filter($settings['title']) . '</' . $heading_tag . '>';
					}
					if( !empty($caption) && (empty($settings['caption-position']) || $settings['caption-position'] == 'bottom') ){
						$ret .= $caption;
					}
					$ret .= '</div>'; // gdlr-core-column-service-title-wrap
				}
				
				// content
				if( !empty($settings['content']) || (!empty($settings['read-more-text']) && !empty($settings['read-more-link'])) ){
					$ret .= '<div class="gdlr-core-column-service-content" ' . gdlr_core_esc_style(array(
						'font-size' => $settings['content-size'],
						'font-weight' => empty($settings['content-font-weight'])? '': $settings['content-font-weight'],
						'text-transform' => empty($settings['content-text-transform'])? '': $settings['content-text-transform'],
					)) . ' >';
					if( !empty($settings['content']) ){
						$ret .= gdlr_core_content_filter($settings['content']);
					}
					if( !empty($settings['read-more-text']) && !empty($settings['read-more-link']) ){
						$ret .= '<a class="gdlr-core-column-service-read-more gdlr-core-info-font" href="' . esc_attr($settings['read-more-link']) . '" ';
						$ret .= empty($settings['read-more-link-target'])? '': ' target="' . esc_attr($settings['read-more-link-target']) . '" ';
						$ret .= gdlr_core_esc_style(array(
							'font-size' => $settings['read-more-size'],
							'font-weight' => empty($settings['read-more-font-weight'])? '': $settings['read-more-font-weight'],
							'font-style' => (empty($settings['read-more-font-style']) || $settings['read-more-font-style'] == 'default')? '': $settings['read-more-font-style'],
							'margin-top' => empty($settings['read-more-top-margin'])? '': $settings['read-more-top-margin']
						)) . ' >';
						$ret .= gdlr_core_escape_content($settings['read-more-text']);
						if( !empty($settings['enable-read-more-icon']) && $settings['enable-read-more-icon'] == 'enable' && !empty($settings['read-more-icon']) ){
							$ret .= '<i class="' . esc_attr($settings['read-more-icon']) . '" ' . gdlr_core_esc_style(array(
								'font-size' => empty($settings['read-more-icon-font-size'])? '': $settings['read-more-icon-font-size']
							)) . ' ></i>';
						}
						$ret .= '</a>';
					}
					$ret .= '</div>'; // gdlr-core-column-service-content
				}
				$ret .= '</div>'; // gdlr-core-column-service-content-wrapper
				$ret .= '</div>'; // gdlr-core-column-service-content-wrapper
				$ret .= $custom_style;

				return $ret;
			}			
			
		} // gdlr_core_pb_element_column_service
	} // class_exists	

	// [gdlr_core_column_service icon="" title="" caption="" style="left_icon-left" ][/gdlr_core_column_service]
	add_shortcode('gdlr_core_column_service', 'gdlr_core_column_service_shortcode');
	if( !function_exists('gdlr_core_column_service_shortcode') ){
		function gdlr_core_column_service_shortcode($atts, $content = ''){
			$atts = wp_parse_args($atts, array(
				'no-pdlr' => true, 
				'style' => 'left_icon-left'
			));

			$atts['content'] = $content;

			return gdlr_core_pb_element_column_service::get_content($atts);
		}
	}