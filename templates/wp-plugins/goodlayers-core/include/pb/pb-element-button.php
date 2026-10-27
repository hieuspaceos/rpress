<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('button', 'gdlr_core_pb_element_button'); 
	
	if( !class_exists('gdlr_core_pb_element_button') ){
		class gdlr_core_pb_element_button{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-square-o',
					'title' => esc_html__('Button', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('Button 1', 'goodlayers-core'),
						'options' => array(
							'button-text' => array(
								'title' => esc_html__('Button Text', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Learn More', 'goodlayers-core'),
							),
							'link-to' => array(
								'title' => esc_html__('Button Link To', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'custom-url' => esc_html__('Custom Url', 'goodlayers-core'),
									'lb-custom-image' => esc_html__('Lightbox with custom image', 'goodlayers-core'),
									'lb-video' => esc_html__('Video Lightbox', 'goodlayers-core'),
								),
								'default' => 'custom-url'
							),
							'custom-image' => array(
								'title' => esc_html__('Upload Custom Image', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'link-to' => 'lb-custom-image' )
							),
							'video-url' => array(
								'title' => esc_html__('Video Url ( Youtube / Vimeo )', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'link-to' => 'lb-video' )
							),					
							'button-link' => array(
								'title' => esc_html__('Button Link', 'goodlayers-core'),
								'type' => 'text',
								'default' => '#',
								'condition' => array( 'link-to' => 'custom-url' )
							),
							'button-link-target' => array(
								'title' => esc_html__('Button Link Target', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'_self' => esc_html__('Current Screen', 'goodlayers-core'),
									'_blank' => esc_html__('New Window', 'goodlayers-core'),
								),
								'default' => '_self',
								'condition' => array( 'link-to' => 'custom-url' )
							),
							'icon-position' => array(
								'title' => esc_html__('Icon Position', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'left' => esc_html__('Left', 'goodlayers-core'),
									'right' => esc_html__('Right', 'goodlayers-core'),
								)
							),
							'icon' => array(
								'title' => esc_html__('Icon Selector', 'goodlayers-core'),
								'type' => 'icons',
								'default' => 'fa fa-gear',
								'wrapper-class' => 'gdlr-core-fullsize',
								'condition' => array( 'icon-position' => array('left', 'right') )
							),
							'z-index' => array(
								'title' => esc_html__('Z Index', 'goodlayers-core'),
								'type' => 'text'
							)
						)
					),
					'style' => array(
						'title' => esc_html__('Button Style', 'goodlayers-core'),
						'options' => array(
							'text-align' => array(
								'title' => esc_html__('Button Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center'
							),
							'mobile-alignment' => array(
								'title' => esc_html__('Mobile Alignment', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('Default', 'goodlayers-core'),
									'left' => esc_html__('Left', 'goodlayers-core'),
									'center' => esc_html__('Center', 'goodlayers-core'),
									'right' => esc_html__('Right', 'goodlayers-core'),
								)
							),
							'full-width-button' => array(
								'title' => esc_html__('Full Width Button', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'button-padding' => array(
								'title' => esc_html__('Button Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'15px', 'right'=>'33px', 'bottom'=>'15px', 'left'=>'33px', 'settings'=>'unlink' )
							),
							'border-radius' => array(
								'title' => esc_html__('Button Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '25px'
							),
							'button-background' => array(
								'title' => esc_html__('Button Background', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'transparent' => esc_html__('Transparent', 'goodlayers-core'),
									'solid' => esc_html__('Solid', 'goodlayers-core'),
									'gradient' => esc_html__('Gradient - Vertical', 'goodlayers-core'),
									'gradient-v' => esc_html__('Gradient - Horizontal', 'goodlayers-core'),
									'bottom-border-on-text' => esc_html__('Bottom Border On Text', 'goodlayers-core')
								),
								'default' => 'gradient'
							),
							'button-border' => array(
								'title' => esc_html__('Button Border', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'button-background' => array('transparent', 'solid', 'gradient', 'gradient-v'), )
							),
							'border-width' => array(
								'title' => esc_html__('Button Border Width', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'1px', 'right'=>'1px', 'bottom'=>'1px', 'left'=>'1px', 'settings'=>'link' ),
								'condition' => array( 'button-background' => array('transparent', 'solid', 'gradient', 'gradient-v'), 'button-border' => 'enable' )
							),
							'border-on-text-size' => array(
								'title' => esc_html__('Button Border Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'button-background' => 'bottom-border-on-text' )
							),
							'border-on-text-top-margin' => array(
								'title' => esc_html__('Button Border Top Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'button-background' => 'bottom-border-on-text' ),
								'description' => esc_html__('Can be negative value', 'goodlayers-core')
							),
							'text-color' => array(
								'title' => esc_html__('Button Text Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'icon-color' => array(
								'title' => esc_html__('Button Icon Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'text-hover-color' => array(
								'title' => esc_html__('Button Text Hover Color', 'goodlayers-core'), 
								'type' => 'colorpicker'
							),
							'icon-hover-color' => array(
								'title' => esc_html__('Button Icon Hover Color', 'goodlayers-core'), 
								'type' => 'colorpicker'
							),
							'background-color' => array(
								'title' => esc_html__('Button Background Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'button-background' => array('solid', 'gradient', 'gradient-v') ) 
							),
							'background-hover-color' => array(
								'title' => esc_html__('Button Background Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'button-background' => array('solid') ) 
							),
							'background-gradient-color' => array(
								'title' => esc_html__('Button Background Gradient Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'button-background' => array('gradient', 'gradient-v') ) 
							),
							'border-color' => array(
								'title' => esc_html__('Button Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'button-background' => array('transparent', 'solid', 'gradient', 'gradient-v'), 'button-border' => 'enable' )
							),
							'border-hover-color' => array(
								'title' => esc_html__('Button Border Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'button-background' => array('transparent', 'solid', 'gradient', 'gradient-v'), 'button-border' => 'enable' )
							),
							'border-on-text-color' => array(
								'title' => esc_html__('Button Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'button-background' => 'bottom-border-on-text' )
							),
							'button-shadow-size' => array(
								'title' => esc_html__('Button Shadow Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
							),
							'button-shadow-color' => array(
								'title' => esc_html__('Button Shadow Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'button-shadow-opacity' => array(
								'title' => esc_html__('Button Shadow Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
							'button-skewx' => array(
								'title' => esc_html__('Skew X', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'description' => esc_html__('Only input number here', 'goodlayers-core')
							)
						)
					),
					'button-2' => array(
						'title' => esc_html__('Button 2', 'goodlayers-core'),
						'options' => array(
							'button2-text' => array(
								'title' => esc_html__('Button 2 Text', 'goodlayers-core'),
								'type' => 'text'
							),
							'button2-link-to' => array(
								'title' => esc_html__('Button 2 Link To', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'custom-url' => esc_html__('Custom Url', 'goodlayers-core'),
									'lb-custom-image' => esc_html__('Lightbox with custom image', 'goodlayers-core'),
									'lb-video' => esc_html__('Video Lightbox', 'goodlayers-core'),
								),
								'default' => 'custom-url'
							),
							'button2-custom-image' => array(
								'title' => esc_html__('Upload Custom Image', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'button2-link-to' => 'lb-custom-image' )
							),
							'button2-video-url' => array(
								'title' => esc_html__('Video Url ( Youtube / Vimeo )', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'button2-link-to' => 'lb-video' )
							),					
							'button2-button-link' => array(
								'title' => esc_html__('Button 2 Link', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'button2-link-to' => 'custom-url' )
							),
							'button2-link-target' => array(
								'title' => esc_html__('Button 2 Link Target', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'_self' => esc_html__('Current Screen', 'goodlayers-core'),
									'_blank' => esc_html__('New Window', 'goodlayers-core'),
								),
								'default' => '_self',
								'condition' => array( 'button2-link-to' => 'custom-url' )
							),
							'button2-icon-position' => array(
								'title' => esc_html__('Button 2 Icon Position', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'left' => esc_html__('Left', 'goodlayers-core'),
									'right' => esc_html__('Right', 'goodlayers-core'),
								)
							),
							'button2-icon' => array(
								'title' => esc_html__('Button 2 Icon Selector', 'goodlayers-core'),
								'type' => 'icons',
								'default' => 'fa fa-gear',
								'wrapper-class' => 'gdlr-core-fullsize',
								'condition' => array( 'button2-icon-position' => array('left', 'right') )
							),
						)
					),
					'style-2' => array(
						'title' => esc_html__('Button 2 Style', 'goodlayers-core'),
						'options' => array(
							'button2-padding' => array(
								'title' => esc_html__('Button Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'15px', 'right'=>'33px', 'bottom'=>'15px', 'left'=>'33px', 'settings'=>'unlink' )
							),
							'border2-radius' => array(
								'title' => esc_html__('Button Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '25px'
							),
							'button2-background' => array(
								'title' => esc_html__('Button Background', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'transparent' => esc_html__('Transparent', 'goodlayers-core'),
									'solid' => esc_html__('Solid', 'goodlayers-core'),
									'gradient' => esc_html__('Gradient - Vertical', 'goodlayers-core'),
									'gradient-v' => esc_html__('Gradient - Horizontal', 'goodlayers-core'),
									'bottom-border-on-text' => esc_html__('Bottom Border On Text', 'goodlayers-core')
								),
								'default' => 'gradient'
							),
							'button2-border' => array(
								'title' => esc_html__('Button Border', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'button2-background' => array('transparent', 'solid', 'gradient', 'gradient-v'), )
							),
							'border2-width' => array(
								'title' => esc_html__('Button Border Width', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'1px', 'right'=>'1px', 'bottom'=>'1px', 'left'=>'1px', 'settings'=>'link' ),
								'condition' => array( 'button2-background' => array('transparent', 'solid', 'gradient', 'gradient-v'), 'button2-border' => 'enable' )
							),
							'border2-on-text-size' => array(
								'title' => esc_html__('Button Border Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'button2-background' => 'bottom-border-on-text' )
							),
							'border2-on-text-top-margin' => array(
								'title' => esc_html__('Button Border Top Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'button2-background' => 'bottom-border-on-text' ),
								'description' => esc_html__('Can be negative value', 'goodlayers-core')
							),
							'text2-color' => array(
								'title' => esc_html__('Button Text Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'icon2-color' => array(
								'title' => esc_html__('Button Icon Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'text2-hover-color' => array(
								'title' => esc_html__('Button Text Hover Color', 'goodlayers-core'), 
								'type' => 'colorpicker'
							),
							'icon2-hover-color' => array(
								'title' => esc_html__('Button Icon Hover Color', 'goodlayers-core'), 
								'type' => 'colorpicker'
							),
							'background2-color' => array(
								'title' => esc_html__('Button Background Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'button2-background' => array('solid', 'gradient', 'gradient-v') ) 
							),
							'background2-hover-color' => array(
								'title' => esc_html__('Button Background Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'button2-background' => array('solid') ) 
							),
							'background2-gradient-color' => array(
								'title' => esc_html__('Button Background Gradient Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'button2-background' => array('gradient', 'gradient-v') ) 
							),
							'border2-color' => array(
								'title' => esc_html__('Button Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'button2-border' => 'enable' )
							),
							'border2-hover-color' => array(
								'title' => esc_html__('Button Border Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'button2-border' => 'enable' )
							),
							'border2-on-text-color' => array(
								'title' => esc_html__('Button Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'button2-background' => 'bottom-border-on-text' )
							),
							'button2-shadow-size' => array(
								'title' => esc_html__('Button Shadow Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
							),
							'button2-shadow-color' => array(
								'title' => esc_html__('Button Shadow Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'button2-shadow-opacity' => array(
								'title' => esc_html__('Button Shadow Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
							'button2-skewx' => array(
								'title' => esc_html__('Skew X', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'description' => esc_html__('Only input number here', 'goodlayers-core')
							)
						)
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'text-size' => array(
								'title' => esc_html__('Button Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'text-font-style' => array(
								'title' => esc_html__('Button Font Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'normal' => esc_html__('Normal', 'goodlayers-core'),
									'italic' => esc_html__('Italic', 'goodlayers-core'),
								),
								'default' => 'normal'
							),
							'text-font-weight' => array(
								'title' => esc_html__('Buttton Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'default' => '',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'text-letter-spacing' => array(
								'title' => esc_html__('Button Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => ''
							),
							'text-transform' => array(
								'title' => esc_html__('Button Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'data-type' => 'text',
								'options' => array(
									'' => esc_html__('Default', 'goodlayers-core'),
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
							),
							'icon-size' => array(
								'title' => esc_html__('Button Icon Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
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
							'button2-left-margin' => array(
								'title' => esc_html__('Button 2 Left Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '10px'
							),
							'item-top-margin' => array(
								'title' => esc_html__('Top Margin ( Item )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '0px'
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
				$content  = self::get_content($settings, true);
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'button-text' => esc_html__('Learn More', 'goodlayers-core'),
						'button-link' => '#', 'button-link-target' => '_self',
						'text-align' => 'center', 'text-size' => '', 'border-radius' => '25px', 'button-padding' => '', 'button-background' => 'gradient', 
						'text-color' => '', 'text-hover-color' => '', 
						'background-color' => '', 'background-hover-color' => '', 'background-gradient-color' => '',
						'button-border' => 'disable', 'border-width' => '', 'border-color' => '', 'border-hover-color' => '',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				if( !empty($settings['button2-text']) ){
					$button_margin = empty($settings['button2-left-margin'])? '10px': $settings['button2-left-margin'];

					if( !empty($settings['text-align']) && $settings['text-align'] == 'center' ){
						$settings['button-margin'] = '0px ' . $button_margin . ' 10px';
					}else{
						$settings['button-margin'] = '0px ' . $button_margin . ' 10px 0px';
					}
					
				}

				// start printing item
				$extra_class  = 'gdlr-core-' . (empty($settings['text-align'])? 'left': $settings['text-align']) . '-align'; 
				if( !empty($settings['mobile-alignment']) ){
					$extra_class .= ' gdlr-core-mobile-' . $settings['mobile-alignment'] . '-align'; 
				}
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$ret  = '<div class="gdlr-core-button-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ' . gdlr_core_esc_style(array(
					'padding-bottom' => (!empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb)? $settings['padding-bottom']: '',
					'margin-top' => (empty($settings['item-top-margin']) || $settings['item-top-margin'] == '0px')? '': $settings['item-top-margin'],
					'transform' => empty($settings['3d-content-z-pos'])? '': 'translateZ(' . $settings['3d-content-z-pos'] . ')'
				));
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				$ret .= gdlr_core_get_button($settings, $preview);

				// button 2
				if( !empty($settings['button2-text']) ){
					$settings['button-text'] = empty($settings['button2-text'])? '': $settings['button2-text']; 
						unset($settings['button2-text']);
					$settings['link-to'] = empty($settings['button2-link-to'])? '': $settings['button2-link-to']; 
						unset($settings['button2-link-to']);
					$settings['custom-image'] = empty($settings['button2-custom-image'])? '': $settings['button2-custom-image']; 
						unset($settings['button2-custom-image']);
					$settings['video-url'] = empty($settings['button2-video-url'])? '': $settings['button2-video-url']; 
						unset($settings['button2-video-url']);
					$settings['button-link'] = empty($settings['button2-button-link'])? '': $settings['button2-button-link']; 
						unset($settings['button2-button-link']);
					$settings['button-link-target'] = empty($settings['button2-link-target'])? '': $settings['button2-link-target']; 
						unset($settings['button2-link-target']);
					$settings['icon-position'] = empty($settings['button2-icon-position'])? '': $settings['button2-icon-position']; 
						unset($settings['button2-icon-position']);
					$settings['icon'] = empty($settings['button2-icon'])? '': $settings['button2-icon']; 
						unset($settings['button2-icon']);
					$settings['button-padding'] = empty($settings['button2-padding'])? '': $settings['button2-padding']; 
						unset($settings['button2-padding']);
					$settings['border-radius'] = empty($settings['border2-radius'])? '': $settings['border2-radius']; 
						unset($settings['border2-radius']);
					$settings['button-background'] = empty($settings['button2-background'])? '': $settings['button2-background']; 
						unset($settings['button2-background']);
					$settings['button-border'] = empty($settings['button2-border'])? '': $settings['button2-border']; 
						unset($settings['button2-border']);
					$settings['border-width'] = empty($settings['border2-width'])? '': $settings['border2-width']; 
						unset($settings['border2-width']);
					$settings['text-color'] = empty($settings['text2-color'])? '': $settings['text2-color']; 
						unset($settings['text2-color']);
					$settings['icon-color'] = empty($settings['icon2-color'])? '': $settings['icon2-color']; 
						unset($settings['icon2-color']);
					$settings['text-hover-color'] = empty($settings['text2-hover-color'])? '': $settings['text2-hover-color']; 
						unset($settings['text2-hover-color']);
					$settings['icon-hover-color'] = empty($settings['icon2-hover-color'])? '': $settings['icon2-hover-color']; 
						unset($settings['icon2-hover-color']);
					$settings['background-color'] = empty($settings['background2-color'])? '': $settings['background2-color']; 
						unset($settings['background2-color']);
					$settings['background-hover-color'] = empty($settings['background2-hover-color'])? '': $settings['background2-hover-color']; 
						unset($settings['background2-hover-color']);
					$settings['background-gradient-color'] = empty($settings['background2-gradient-color'])? '': $settings['background2-gradient-color']; 
						unset($settings['background2-gradient-color']);
					$settings['border-color'] = empty($settings['border2-color'])? '': $settings['border2-color']; 
						unset($settings['border2-color']);
					$settings['border-hover-color'] = empty($settings['border2-hover-color'])? '': $settings['border2-hover-color']; 
						unset($settings['border2-hover-color']);
					$settings['button-shadow-size'] = empty($settings['button2-shadow-size'])? '': $settings['button2-shadow-size']; 
						unset($settings['button2-shadow-size']);
					$settings['button-shadow-color'] = empty($settings['button2-shadow-color'])? '': $settings['button2-shadow-color']; 
						unset($settings['button2-shadow-color']);
					$settings['button-shadow-opacity'] = empty($settings['button2-shadow-opacity'])? '': $settings['button2-shadow-opacity']; 
						unset($settings['button2-shadow-opacity']);

					$settings['button-skewx'] = empty($settings['button2-skewx'])? '': $settings['button2-skewx']; 
						unset($settings['button2-skewx']);
					
					$settings['border-on-text-size'] = empty($settings['border2-on-text-size'])? '': $settings['border2-on-text-size'];
						unset($settings['border2-on-text-size']);
					$settings['border-on-text-top-margin'] = empty($settings['border2-on-text-top-margin'])? '': $settings['border2-on-text-top-margin'];
						unset($settings['border2-on-text-top-margin']);
					$settings['border-on-text-color'] = empty($settings['border2-on-text-color'])? '': $settings['border2-on-text-color'];
						unset($settings['border2-on-text-color']);

					unset($settings['button-margin']);

					$ret .= gdlr_core_get_button($settings, $preview);
				}
				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_content
	} // class_exists	
	
	if( !function_exists('gdlr_core_get_button') ){
		function gdlr_core_get_button( $atts, $preview = false ){
			
			$ret  = '';
			$extra_class = empty($atts['extra-class'])? '': $atts['extra-class'];
			$skewx = empty($atts['button-skewx'])? '': intval($atts['button-skewx']);

			$css_atts = array(
				'font-size' => (empty($atts['text-size']))? '': $atts['text-size'],
				'font-style' => empty($atts['text-font-style'])? '': $atts['text-font-style'],
				'font-weight' => empty($atts['text-font-weight'])? '': $atts['text-font-weight'],
				'letter-spacing' => empty($atts['text-letter-spacing'])? '': $atts['text-letter-spacing'],
				'color' => empty($atts['text-color'])? '': $atts['text-color'],
				'padding' => (empty($atts['button-padding']) || $atts['button-padding'] == array(
						'top' => '15px', 'right' => '33px', 'bottom' => '15px', 'left' => '33px', 'settings' => 'unlink'
					))? '': $atts['button-padding'],
				'text-transform' => empty($atts['text-transform'])? '': $atts['text-transform'], 
				'margin' => empty($atts['button-margin'])? '': $atts['button-margin'], 
				'border-radius' => (empty($atts['border-radius']) || $atts['border-radius'] == '25px')? '': $atts['border-radius'],
				'background-shadow-size' => empty($atts['button-shadow-size'])? '': $atts['button-shadow-size'],
				'background-shadow-color' => empty($atts['button-shadow-color'])? '': $atts['button-shadow-color'],
				'background-shadow-opacity' => empty($atts['button-shadow-opacity'])? '': $atts['button-shadow-opacity'],
				'position' => (empty($atts['position']))? '': $atts['position'],
				'skewx' => $skewx
			);
			$css_icon_atts = array(
				'font-size' => empty($atts['icon-size'])? '': $atts['icon-size'],
				'color' => empty($atts['icon-color'])? '': $atts['icon-color']
			);

			if( !empty($atts['z-index']) ){
				$css_atts['z-index'] = $atts['z-index'];
				$css_atts['position'] = 'relative';
 			}
			
			// background
			$atts['button-background'] = empty($atts['button-background'])? 'gradient': $atts['button-background'];
			$extra_class .= ' gdlr-core-button-' . $atts['button-background'];
			$extra_class .= empty($atts['text-align'])? '': ' gdlr-core-' . $atts['text-align'] . '-align';
			if( $atts['button-background'] == 'solid' ){
				$css_atts['background'] = empty($atts['background-color'])? '': $atts['background-color'];
			}else if( in_array($atts['button-background'], array('gradient', 'gradient-v')) ){
				if( !empty($atts['background-color']) ){
					$css_atts['background'] = $atts['background-color'];
					
					if( !empty($atts['background-gradient-color']) ){
						$css_atts[$atts['button-background']] = array(
							$atts['background-color'],
							$atts['background-gradient-color']
						);
					}
				}
			}
			
			// border
			if( $atts['button-background'] == 'bottom-border-on-text' ||
				(empty($atts['button-border']) && $atts['button-background'] != 'transparent' ) || 
				(!empty($atts['button-border']) && $atts['button-border'] == 'disable') ){	

				$extra_class .= ' gdlr-core-button-no-border';
			}else{
				$extra_class .= ' gdlr-core-button-with-border';
				$css_atts['border-width'] = (empty($atts['border-width']) || $atts['border-width'] == array( 
						'top'=>'1px', 'right'=>'1px', 'bottom'=>'1px', 'left'=>'1px', 'settings'=>'link'
					))? '': $atts['border-width'];
				$css_atts['border-color'] = empty($atts['border-color'])? '': $atts['border-color'];
			}
			
			if( !empty($atts['full-width-button']) && $atts['full-width-button'] == 'enable' ){
				$extra_class .= ' gdlr-core-button-full-width';
			} 

			// hover css for 'text-hover-color''background-hover-color''border-hover-color'
			$hover_style = gdlr_core_esc_style(array(
				'color' => empty($atts['text-hover-color'])? '': $atts['text-hover-color'],
				'border-color' => empty($atts['border-hover-color'])? '': $atts['border-hover-color'],
				'background-color' => (!empty($atts['background-hover-color']) && $atts['button-background'] == 'solid')? $atts['background-hover-color']: ''
			), false);
			$icon_hover_style =  gdlr_core_esc_style(array(
				'color' => empty($atts['icon-hover-color'])? '': $atts['icon-hover-color'],
			), false);
			
			if( !empty($hover_style) || !empty($icon_hover_style) ){
				global $gdlr_core_button_id; 

				if( $preview ){
					$gdlr_core_button_id = empty($gdlr_core_button_id) || !is_array($gdlr_core_button_id)? array(): $gdlr_core_button_id;
				
					// generate unique id so it does not get overwritten in admin area
					$rnd_button_id = mt_rand(0, 99999);
					while( in_array($rnd_button_id, $gdlr_core_button_id) ){
						$rnd_button_id = mt_rand(0, 99999);
					}
					$gdlr_core_button_id[] = $rnd_button_id;
				}else{
					$gdlr_core_button_id = empty($gdlr_core_button_id)? 1: $gdlr_core_button_id + 1;
					$rnd_button_id = $gdlr_core_button_id;
				}
				
				$extra_style = '';
				if( !empty($hover_style) ){
					$extra_style .= '#gdlr-core-button-id-' . $rnd_button_id . '{' . gdlr_core_esc_style($css_atts, false) . '}';
					$extra_style .= '#gdlr-core-button-id-' . $rnd_button_id . ':hover{' . $hover_style . '}';
					$css_atts = array();
				}
				if( !empty($icon_hover_style) ){
					$extra_style .= '#gdlr-core-button-id-' . $rnd_button_id . ' i{' . gdlr_core_esc_style($css_icon_atts, false) . '}';
					$extra_style .= '#gdlr-core-button-id-' . $rnd_button_id . ':hover i{' . $icon_hover_style . '}';
					$css_icon_atts = array();
				}
				
				if( $preview ){
					$ret .= '<style>' . $extra_style . '</style>';
				}else{
					gdlr_core_add_inline_style($extra_style);
				}
				
			}
			
			// printing item
			if( !empty($atts['button-text']) ){

				if( !empty($atts['icon-position']) && $atts['icon-position'] != 'none' && !empty($atts['icon']) ){
					$button_icon = '<i class="gdlr-core-pos-' . esc_attr($atts['icon-position']) . ' ' . $atts['icon'] . '" ' . gdlr_core_esc_style($css_icon_atts) . ' ></i>';
				}
				$ret .= '<a ';
				if( empty($atts['link-to']) || $atts['link-to'] == 'custom-url' ){
					$ret .= 'class="gdlr-core-button ' . esc_attr($extra_class) . '" href="' . esc_url($atts['button-link']) . '" ';
					$ret .= (empty($atts['button-link-target']) || $atts['button-link-target'] == '_self')? '': 'target="' . esc_attr($atts['button-link-target']) . '" ';
				}else if( $atts['link-to'] == 'lb-content' ){
					global $gdlr_core_lb_content_id;
					$gdlr_core_lb_content_id = empty($gdlr_core_lb_content_id)? 1: $gdlr_core_lb_content_id + 1;

					$ret .= 'class="gdlr-core-button ' . esc_attr($extra_class) . ' gdlr-core-ilightbox gdlr-core-js" ';
					$ret .= 'data-type="inline" href="#gdlr-core-lb-content-' . esc_attr($gdlr_core_lb_content_id) . '" ';
				}else if( $atts['link-to'] == 'lb-custom-image' ){
					$image_url = '';
					$caption = '';
					if( !empty($atts['custom-image']) ){
						if( is_numeric($atts['custom-image']) ){
							$image_url = gdlr_core_get_image_url($atts['custom-image']);
							$caption = gdlr_core_get_image_info($atts['custom-image'], 'caption');;
						}else{
							$image_url = $atts['custom-image'];
						}
					}

					$ret .= gdlr_core_get_lightbox_atts(array(
						'class'=>'gdlr-core-button ' . $extra_class,
						'url'=>$image_url,
						'caption'=>$caption
					));
				}else if( $atts['link-to'] == 'lb-video' ){
					$ret .= gdlr_core_get_lightbox_atts(array(
						'class'=>'gdlr-core-button ' . $extra_class,
						'url'=>$atts['video-url'],
						'type'=>'video'
					));
				}
				$ret .= empty($rnd_button_id)? ' ': ' id="gdlr-core-button-id-' . esc_attr($rnd_button_id) . '" ';
				$ret .= gdlr_core_esc_style($css_atts) . ' >';
				if( $atts['button-background'] == 'bottom-border-on-text' ){
					$ret .= '<span class="gdlr-core-border-on-text" ' . gdlr_core_esc_style(array(
						'border-bottom-width' => empty($atts['border-on-text-size'])? '': $atts['border-on-text-size'],
						'border-color' => empty($atts['border-on-text-color'])? '': $atts['border-on-text-color'],
						'margin-top' => empty($atts['border-on-text-top-margin'])? '': $atts['border-on-text-top-margin']
					)) . ' ></span>';
				}

				$content_atts = array(
					'skewx' => -1 * intval($skewx)
				);
				if( !empty($button_icon) && $atts['icon-position'] == 'left' ){
					$ret .= '<span class="gdlr-core-content" ' . gdlr_core_esc_style($content_atts) . ' >' . $button_icon . gdlr_core_escape_content($atts['button-text']) . '</span>';
				}else if( !empty($button_icon) && $atts['icon-position'] == 'right' ){
					$ret .= '<span class="gdlr-core-content" ' . gdlr_core_esc_style($content_atts) . ' >' . gdlr_core_escape_content($atts['button-text']) . $button_icon . '</span>' ;
				}else{
					$ret .= '<span class="gdlr-core-content" ' . gdlr_core_esc_style($content_atts) . ' >' . gdlr_core_escape_content($atts['button-text']) . '</span>';
				}
				$ret .= '</a>';

				if( !empty($atts['lb-content']) && !empty($gdlr_core_lb_content_id) ){
					$ret .= '<div class="gdlr-core-lb-content" id="gdlr-core-lb-content-' . esc_attr($gdlr_core_lb_content_id) . '" style="display: none" >';
					$ret .= $atts['lb-content'];
					$ret .= '</div>';
				}
			}
			
			return $ret;
		}
	}
	
	// [gdlr_core_button button-text="Learn More" button-link="#" button-link-target="_blank" margin-right="20px" ]
	add_shortcode('gdlr_core_button', 'gdlr_core_button_shortcode');
	if( !function_exists('gdlr_core_button_shortcode') ){
		function gdlr_core_button_shortcode($atts, $content = ''){
			$atts = wp_parse_args($atts, array(
				'no-pdlr' => true,
				'margin-right' => '20px',
				'button-text' => 'Learn More',
				'extra-class' => 'gdlr-core-button-shortcode '
			));

			$atts['button-margin'] = array(
				'right' => (empty($atts['margin-right']) || $atts['margin-right'] == '0px')? '': $atts['margin-right'],
				'left' => (empty($atts['margin-left']) || $atts['margin-left'] == '0px')? '': $atts['margin-left'],
				'bottom' => (empty($atts['margin-bottom']) || $atts['margin-bottom'] == '0px')? '': $atts['margin-bottom']
			);
			$atts['button-padding'] = empty($atts['padding'])? '': $atts['padding'];

			if( !empty($atts['border-width']) ){
				$atts['button-border'] = 'enable';
			}

			if( !empty($content) ){

				$lightbox_type = apply_filters('gdlr_core_lightbox_type', 'strip');
				if( strpos($lightbox_type, 'ilightbox') === false ){
					return esc_html__('Please set the lightbox type to be "ilightbox" to use this feature', 'goodlayers-core');
				}else{
					$atts['lb-content'] = $content;
					$atts['link-to'] = 'lb-content';
				}

			}else if( !empty($atts['video-url']) ){
				$atts['link-to'] = 'lb-video';
			}else if( !empty($atts['custom-image']) ){
				$atts['link-to'] = 'lb-custom-image';
			}

			return gdlr_core_get_button($atts);
		}
	}