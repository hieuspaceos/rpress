<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('hover-box', 'gdlr_core_pb_element_hover_box'); 
	
	if( !class_exists('gdlr_core_pb_element_hover_box') ){
		class gdlr_core_pb_element_hover_box{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-inbox',
					'title' => esc_html__('Hover Box', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'tabs' => array(
								'title' => esc_html__('Add New Tab', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'tabs',
								'wrapper-class' => 'gdlr-core-fullsize',
								'options' => array(
									'image' => array(
										'title' => esc_html__('Image', 'goodlayers-core'),
										'type' => 'upload'
									),
									'image-hover' => array(
										'title' => esc_html__('Image Hover', 'goodlayers-core'),
										'type' => 'upload'
									),
									'title' => array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'content' => array(
										'title' => esc_html__('Content', 'goodlayers-core'),
										'type' => 'tmce'
									),
									'link' => array(
										'title' => esc_html__('Link', 'goodlayers-core'),
										'type' => 'text'
									),
									'background' => array(
										'title' => esc_html__('Background', 'goodlayers-core'),
										'type' => 'upload'
									),
									'background-hover' => array(
										'title' => esc_html__('Background Hover', 'goodlayers-core'),
										'type' => 'upload'
									)
								),
								'default' => array(
									array(
										'title' => esc_html__('Sameple Name', 'goodlayers-core'),
										'content' => esc_html__('Sample content area', 'goodlayers-core'),
									),
									array(
										'title' => esc_html__('Sameple Name', 'goodlayers-core'),
										'content' => esc_html__('Sample content area', 'goodlayers-core'),
									),
								)
							),
						),
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'style' => array(
								'title' => esc_html__('Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'top-image' => esc_html__('Top Image', 'goodlayers-core'),
									'left-image' => esc_html__('Left Image', 'goodlayers-core'),
								)
							),
							'text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center',
								'condition' => array('style' => 'top-image' )
							),	
							'image-width' => array(
								'title' => esc_html__('Image Width', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'image-size' => array(
								'title' => esc_html__('Image Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'thumbnail-size',
								'default' => 'full'
							),
							'image-position' => array(
								'title' => esc_html__('Image Position', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'top' => esc_html__('Top of Hover Box', 'goodlayers-core'),
									'inside' => esc_html__('Inside Hover Box', 'goodlayers-core')
								),
								'default' => 'top',
								'condition' => array('style' => 'top-image' )
							),
							'link-type' => array(
								'title' => esc_html__('Link Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'box' => esc_html__('Box', 'goodlayers-core'),
									'learn-more' => esc_html__('Learn More', 'goodlayers-core')
								)
							),
							'column' => array(
								'title' => esc_html__('Column Number', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 1=>1, 2=>2, 3=>3, 4=>4, 5=>5, 6=>6),
								'default' => 3
							),
							'carousel' => array(
								'title' => esc_html__('Enable Carousel', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'carousel-autoslide' => array(
								'title' => esc_html__('Autoslide Carousel', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array( 'carousel' => 'enable' )
							),
							'carousel-scrolling-item-amount' => array(
								'title' => esc_html__('Carousel Scrolling Item Amount', 'goodlayers-core'),
								'type' => 'text',
								'default' => '1',
								'condition' => array( 'carousel' => 'enable' )
							),
							'carousel-navigation' => array(
								'title' => esc_html__('Carousel Navigation', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => (function_exists('gdlr_core_get_flexslider_navigation_types')? gdlr_core_get_flexslider_navigation_types(): array()),
								'default' => 'navigation',
								'condition' => array( 'carousel' => 'enable' )
							),
							'carousel-navigation-show-on-hover' => array(
								'title' => esc_html__('Carousel Navigation Display On Hover', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'carousel-navigation' => array('navigation-outer', 'navigation-inner') )
							),
							'carousel-navigation-align' => (function_exists('gdlr_core_get_flexslider_navigation_align')? gdlr_core_get_flexslider_navigation_align(): array()),
							'carousel-navigation-left-icon' => (function_exists('gdlr_core_get_flexslider_navigation_left_icon')? gdlr_core_get_flexslider_navigation_left_icon(): array()),
							'carousel-navigation-right-icon' => (function_exists('gdlr_core_get_flexslider_navigation_right_icon')? gdlr_core_get_flexslider_navigation_right_icon(): array()),
							'carousel-navigation-size' => (function_exists('gdlr_core_get_flexslider_navigation_icon_size')? gdlr_core_get_flexslider_navigation_icon_size(): array()),
							'carousel-navigation-icon-color' => (function_exists('gdlr_core_get_flexslider_navigation_icon_color')? gdlr_core_get_flexslider_navigation_icon_color(): array()),
							'carousel-navigation-icon-hover-color' => (function_exists('gdlr_core_get_flexslider_navigation_icon_hover_color')? gdlr_core_get_flexslider_navigation_icon_hover_color(): array()),
							'carousel-navigation-icon-bg' => (function_exists('gdlr_core_get_flexslider_navigation_icon_background')? gdlr_core_get_flexslider_navigation_icon_background(): array()),
							'carousel-navigation-icon-padding' => (function_exists('gdlr_core_get_flexslider_navigation_icon_padding')? gdlr_core_get_flexslider_navigation_icon_padding(): array()),
							'carousel-navigation-icon-radius' => (function_exists('gdlr_core_get_flexslider_navigation_icon_radius')? gdlr_core_get_flexslider_navigation_icon_radius(): array()),
							'carousel-navigation-margin' => (function_exists('gdlr_core_get_flexslider_navigation_margin')? gdlr_core_get_flexslider_navigation_margin(): array()),
							'carousel-navigation-side-margin' => (function_exists('gdlr_core_get_flexslider_navigation_side_margin')? gdlr_core_get_flexslider_navigation_side_margin(): array()),
							'carousel-navigation-icon-margin' => (function_exists('gdlr_core_get_flexslider_navigation_icon_margin')? gdlr_core_get_flexslider_navigation_icon_margin(): array()),
							'carousel-bullet-style' => array(
								'title' => esc_html__('Carousel Bullet Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => (function_exists('gdlr_core_get_flexslider_bullet_itypes')? gdlr_core_get_flexslider_bullet_itypes(): array()),
								'condition' => array( 'carousel' => 'enable', 'carousel-navigation' => array('bullet','both') ),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'carousel-bullet-top-margin' => array(
								'title' => esc_html__('Carousel Bullet Top Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'layout' => 'carousel', 'carousel-navigation' => array('bullet','both') )
							),
							/*
							'background-skewx' => array(
								'title' => esc_html__('Background Skew X', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Only input number here.', 'goodlayers-core')
							)	
							*/
						)
					),
					'typography' => array(
						'title' => esc_html__('Typograhy', 'goodlayers-core'),
						'options' => array(
							'hover-box-title-size' => array(
								'title' => esc_html__('Title Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'hover-box-title-text-transform' => array(
								'title' => esc_html__('Title Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'data-type' => 'text',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => 'none'
							),
							'hover-box-title-font-weight' => array(
								'title' => esc_html__('Title Font Weight', 'goodlayers-core'),
								'type' => 'text',
							),
							'hover-box-title-letter-spacing' => array(
								'title' => esc_html__('Title Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'title-bottom-margin' => array(
								'title' => esc_html__('Title Bottom Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'content-size' => array(
								'title' => esc_html__('Content Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'content-font-weight' => array(
								'title' => esc_html__('Content Font Weight', 'goodlayers-core'),
								'type' => 'text',
							),
							'content-bottom-margin' => array(
								'title' => esc_html__('Content Bottom Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
						)
					),				
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'hover-box-title-color' => array(
								'title' => esc_html__('Hover Box Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'hover-box-content-color' => array(
								'title' => esc_html__('Hover Box Content Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'hover-box-text-link-color' => array(
								'title' => esc_html__('Hover Box Text link Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'hover-box-background-color' => array(
								'title' => esc_html__('Hover Box Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'hover-box-border-color' => array(
								'title' => esc_html__('Hover Box Border Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'hover-box-title-hover-color' => array(
								'title' => esc_html__('Hover Box Title Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'hover-box-content-hover-color' => array(
								'title' => esc_html__('Hover Box Content Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'hover-box-text-link-hover-color' => array(
								'title' => esc_html__('Hover Box Text Link Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'hover-box-background-hover-color' => array(
								'title' => esc_html__('Hover Box Background Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'hover-box-border-hover-color' => array(
								'title' => esc_html__('Hover Box Border Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
						)
					),
					'shadow' => array(
						'title' => esc_html__('Shadow', 'goodlayers-core'),
						'options' => array(
							'shadow-size' => array(
								'title' => esc_html__('Shadow Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
							),
							'shadow-color' => array(
								'title' => esc_html__('Shadow Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'shadow-opacity' => array(
								'title' => esc_html__('Shadow Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
							'enable-move-up-shadow-effect' => array(
								'title' => esc_html__('Move Up Shadow Hover Effect', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'description' => esc_html__('Only effects the "Column With Frame" style', 'goodlayers-core')
							),
							'move-up-effect-length' => array(
								'title' => esc_html__('Move Up Hover Effect Length', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'enable-move-up-shadow-effect' => 'enable' )
							),
							'frame-hover-shadow-size' => array(
								'title' => esc_html__('Shadow Hover Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
								'condition' => array( 'enable-move-up-shadow-effect' => 'enable' )
							),
							'frame-hover-shadow-color' => array(
								'title' => esc_html__('Shadow Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'enable-move-up-shadow-effect' => 'enable' )
							),
							'frame-hover-shadow-opacity' => array(
								'title' => esc_html__('Shadow Hover Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core'),
								'condition' => array( 'enable-move-up-shadow-effect' => 'enable' )
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Border/Spacing', 'goodlayers-core'),
						'options' => array(
							'hover-box-border' => array(
								'title' => esc_html__('Hover Box Border Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'unlink' )
							),
							'hover-box-border-radius' => array(
								'title' => esc_html__('Hover Box Border Radius', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'unlink' )
							),
							'hover-box-padding' => array(
								'title' => esc_html__('Hover Box Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'45px', 'right'=>'30px', 'bottom'=>'25px', 'left'=>'30px', 'settings'=>'unlink' )
							),
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom ( Wrapper )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							)
						)
					),
					'item-title' => array(
						'title' => esc_html__('Item Title', 'goodlayers-core'),
						'options' => gdlr_core_block_item_option()
					)
				);
			}

			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-hover-box-<?php echo esc_attr($id); ?>" >
if( document.readyState == 'complete' ){
	jQuery(document).ready(function(){
		jQuery('#gdlr-core-preview-hover-box-<?php echo esc_attr($id); ?>').parent().gdlr_core_flexslider();
	});
}else{
	jQuery(window).on('load', function(){
		jQuery('#gdlr-core-preview-hover-box-<?php echo esc_attr($id); ?>').parent().gdlr_core_flexslider();
	});
}
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
						'tabs' => array(
							array(
								'title' => esc_html__('Sameple Name', 'goodlayers-core'),
								'content' => esc_html__('Sample content area', 'goodlayers-core'),
							),
							array(
								'title' => esc_html__('Sameple Name', 'goodlayers-core'),
								'content' => esc_html__('Sample content area', 'goodlayers-core'),
							),
						),
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				$extra_style = '';

				if( !empty($settings['hover-box-title-hover-color']) ){
					$extra_style .= '#custom_style_id .gdlr-core-hover-box:hover .gdlr-core-hover-box-title{ color: ' . $settings['hover-box-title-hover-color'] . ' !important; }';
				}
				if( !empty($settings['hover-box-content-hover-color']) ){
					$extra_style .= '#custom_style_id .gdlr-core-hover-box:hover .gdlr-core-hover-box-content{ color: ' . $settings['hover-box-content-hover-color'] . ' !important; }';
				}
				if( !empty($settings['hover-box-text-link-hover-color']) ){
					$extra_style .= '#custom_style_id .gdlr-core-hover-box:hover .gdlr-core-hover-box-text-link{ color: ' . $settings['hover-box-text-link-hover-color'] . ' !important; }';
				}
				if( !empty($settings['hover-box-background-hover-color']) ){
					$extra_style .= '#custom_style_id .gdlr-core-hover-box:hover{ background-color: ' . $settings['hover-box-background-hover-color'] . ' !important; }';
				}
				if( !empty($settings['hover-box-border-hover-color']) ){
					$extra_style .= '#custom_style_id .gdlr-core-hover-box:hover{ border-color: ' . $settings['hover-box-border-hover-color'] . ' !important; }';
				}
				if( !empty($settings['carousel-navigation-icon-hover-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-flexslider-custom-nav i:hover{ color: ' . $settings['carousel-navigation-icon-hover-color'] . ' !important; }';
				}
				if( !empty($settings['enable-move-up-shadow-effect']) && $settings['enable-move-up-shadow-effect'] == 'enable' ){
					$custom_style_temp = gdlr_core_esc_style(array(
						'background-shadow-size' => empty($settings['frame-hover-shadow-size'])? '': $settings['frame-hover-shadow-size'],
						'background-shadow-color' => empty($settings['frame-hover-shadow-color'])? '': $settings['frame-hover-shadow-color'],
						'background-shadow-opacity' => empty($settings['frame-hover-shadow-opacity'])? '': $settings['frame-hover-shadow-opacity'],
					), false);

					if( !empty($settings['move-up-effect-length']) ){
						$custom_style_temp .= 'transform: translate3d(0, -' . $settings['move-up-effect-length'] . ', 0); ';
					}

					if( !empty($custom_style_temp) ){
						$extra_style .= '#custom_style_id .gdlr-core-move-up-with-shadow:hover{ ' . $custom_style_temp . ' }';
					}
				}

				if( !empty($extra_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_hover_box_id; 

						if( $preview ){
							$gdlr_core_hover_box_id = empty($gdlr_core_hover_box_id)? array(): $gdlr_core_hover_box_id;

							// generate unique id so it does not get overwritten in admin area
							$rnd_id = mt_rand(0, 99999);
							while( in_array($rnd_id, $gdlr_core_hover_box_id) ){
								$rnd_id = mt_rand(0, 99999);
							}
							$gdlr_core_hover_box_id[] = $rnd_id;

							$settings['id'] = 'gdlr-core-hover-box-id-' . $rnd_id;
						}else{
							$gdlr_core_hover_box_id = empty($gdlr_core_hover_box_id)? 1: $gdlr_core_hover_box_id + 1;
							$settings['id'] = 'gdlr-core-hover-box-id-' . $gdlr_core_hover_box_id;
						}
					}

					$extra_style = str_replace('custom_style_id', $settings['id'], $extra_style); 

					if( $preview ){
						$extra_style = '<style>' . $extra_style . '</style>';
					}else{
						gdlr_core_add_inline_style($extra_style);
						$extra_style = '';
					}
				}
				

				// default value
				$settings['column'] = empty($settings['column'])? '3': $settings['column'];
				$settings['carousel'] = empty($settings['carousel'])? 'disable': $settings['carousel'];
				$settings['image-size'] = empty($settings['image-size'])? 'full': $settings['image-size'];

				// start printing item
				$extra_class  = ($settings['carousel'] == 'enable')? ' gdlr-core-item-pdlr': '';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				
				if( empty($settings['style']) || $settings['style'] == 'top-image' ){
					$settings['image-position'] = empty($settings['image-position'])? 'top': $settings['image-position'];
					$extra_class .= ' gdlr-core-' . (empty($settings['text-align'])? 'center': $settings['text-align']) . '-align';
				}else if( $settings['style'] == 'left-image' ){
					$settings['image-position'] = 'top';
					$extra_class .= ' gdlr-core-style-left-image';
				}
				
				
				$ret  = $extra_style;
				$ret .= '<div class="gdlr-core-hover-box-item gdlr-core-item-pdb clearfix ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				$ret .= gdlr_core_block_item_title($settings);

				// grid item
				if( $settings['carousel'] == 'disable' ){

					if( !empty($settings['tabs']) ){
						$t_column_count = 0;
						$t_column = 60 / intval($settings['column']);
						foreach( $settings['tabs'] as $tab ){
							$column_class  = ' gdlr-core-column-' . $t_column;
							$column_class .= ($t_column_count % 60 == 0)? ' gdlr-core-column-first': '';

							$ret .= '<div class="gdlr-core-hover-box-column gdlr-core-item-pdlr gdlr-core-item-mgb ' . esc_attr($column_class) . '" >';
							$ret .= self::get_tab_item($tab, $settings);
							$ret .= '</div>';

							$t_column_count += $t_column;
						}
					}

				// carousel item
				}else{
					$slides = array();
					$flex_atts = array(
						'carousel' => true,
						'column' => empty($settings['column'])? '3': $settings['column'],
						'move' => empty($settings['carousel-scrolling-item-amount'])? '': $settings['carousel-scrolling-item-amount'],
						'navigation' => empty($settings['carousel-navigation'])? 'navigation': $settings['carousel-navigation'],
						'navigation-on-hover' => empty($settings['carousel-navigation-show-on-hover'])? 'disable': $settings['carousel-navigation-show-on-hover'],
						'navigation-align' => empty($settings['carousel-navigation-align'])? '': $settings['carousel-navigation-align'],
						'navigation-size' => empty($settings['carousel-navigation-size'])? '': $settings['carousel-navigation-size'],
						'navigation-icon-color' => empty($settings['carousel-navigation-icon-color'])? '': $settings['carousel-navigation-icon-color'],
						'navigation-icon-background' => empty($settings['carousel-navigation-icon-bg'])? '': $settings['carousel-navigation-icon-bg'],
						'navigation-icon-padding' => empty($settings['carousel-navigation-icon-padding'])? '': $settings['carousel-navigation-icon-padding'],
						'navigation-icon-radius' => empty($settings['carousel-navigation-icon-radius'])? '': $settings['carousel-navigation-icon-radius'],
						'navigation-margin' => empty($settings['carousel-navigation-margin'])? '': $settings['carousel-navigation-margin'],
						'navigation-side-margin' => empty($settings['carousel-navigation-side-margin'])? '': $settings['carousel-navigation-side-margin'],
						'navigation-icon-margin' => empty($settings['carousel-navigation-icon-margin'])? '': $settings['carousel-navigation-icon-margin'],
						'navigation-left-icon' => empty($settings['carousel-navigation-left-icon'])? '': $settings['carousel-navigation-left-icon'],
						'navigation-right-icon' => empty($settings['carousel-navigation-right-icon'])? '': $settings['carousel-navigation-right-icon'],
						'bullet-style' => empty($settings['carousel-bullet-style'])? '': $settings['carousel-bullet-style'],
						'controls-top-margin' => empty($settings['carousel-bullet-top-margin'])? '': $settings['carousel-bullet-top-margin'],
						'nav-parent' => 'gdlr-core-hover-box-item',
						'disable-autoslide' => (empty($settings['carousel-autoslide']) || $settings['carousel-autoslide'] == 'enable')? '': true,
					);

					if( !empty($settings['tabs']) ){
						foreach( $settings['tabs'] as $tab ){
							$slides[] = self::get_tab_item($tab, $settings);
						}
					}

					$ret .= gdlr_core_get_flexslider($slides, $flex_atts);
				}

				$ret .= '</div>'; // gdlr-core-hover-box-item
				
				return $ret;
			}

			static function get_tab_item( $tab = array(), $settings = array() ){ 

				$item_atts = array(
					'background-color' => empty($settings['hover-box-background-color'])? '': $settings['hover-box-background-color'],
					'border-width' => empty($settings['hover-box-border'])? '': $settings['hover-box-border'],
					'border-color' => empty($settings['hover-box-border-color'])? '': $settings['hover-box-border-color'],
					'border-radius' => empty($settings['hover-box-border-radius'])? '': $settings['hover-box-border-radius']
				);
				$item_class = '';
				$item_class .= (!empty($settings['enable-move-up-shadow-effect']) && $settings['enable-move-up-shadow-effect'] == 'enable')? ' gdlr-core-move-up-with-shadow': '';
				if( !empty($settings['shadow-size']['size']) && !empty($settings['shadow-color']) && !empty($settings['shadow-opacity']) ){
					$item_atts = $item_atts + array(
						'background-shadow-size' => empty($settings['shadow-size'])? '': $settings['shadow-size'],
						'background-shadow-color' => empty($settings['shadow-color'])? '': $settings['shadow-color'],
						'background-shadow-opacity' => empty($settings['shadow-opacity'])? '': $settings['shadow-opacity'],
					);

					$item_class .= ' gdlr-core-outer-frame-element ';
				}

				$content_atts = array();
				if( !empty($settings['style']) && $settings['style'] == 'left-image' ){
					$item_atts['padding'] = (empty($settings['hover-box-padding']) || $settings['hover-box-padding'] == array( 
						'top'=>'45px', 'right'=>'30px', 'bottom'=>'25px', 'left'=>'30px', 'settings'=>'unlink' 
					))? '': $settings['hover-box-padding'];
				}else{
					$content_atts['padding'] = (empty($settings['hover-box-padding']) || $settings['hover-box-padding'] == array( 
						'top'=>'45px', 'right'=>'30px', 'bottom'=>'25px', 'left'=>'30px', 'settings'=>'unlink' 
					))? '': $settings['hover-box-padding'];
				}

				if( !empty($settings['background-skewx']) ){
					$item_class .= ' gdlr-core-outer-frame-element';
					$skew = intval($settings['background-skewx']);

					$item_atts['skewx'] = $skew;
					$content_atts['skewx'] = -1 * $skew;
				}
				$ret  = '<div class="gdlr-core-hover-box gdlr-core-js ' . esc_attr($item_class) . ' clearfix" ' . gdlr_core_esc_style($item_atts) . ' >';
				if( !empty($tab['background']) ){
					$ret .= '<div class="gdlr-core-hover-box-bg" ' . gdlr_core_esc_style(array(
						'background-image' => $tab['background']
					)) . ' ></div>';
				}
				if( !empty($tab['background-hover']) ){
					$ret .= '<div class="gdlr-core-hover-box-bg-hover" ' . gdlr_core_esc_style(array(
						'background-image' => $tab['background-hover']
					)) . ' ></div>';
				}
				if( $settings['image-position'] == 'top' && !empty($tab['image']) ){
					$tab['image-hover'] = empty($tab['image-hover'])? '': $tab['image-hover'];

					$ret .= '<div class="gdlr-core-hover-box-thumbnail-top gdlr-core-media-image ';
					$ret .= (empty($tab['image-hover'])? '': 'gdlr-core-hover-thumbnail ');
					$ret .= '" ' . gdlr_core_esc_style(array(
						'width' => empty($settings['image-width'])? '': $settings['image-width']
					)) . ' >';
					$ret .= gdlr_core_get_image($tab['image'], $settings['image-size']);
					$ret .= empty($tab['image-hover'])? '': gdlr_core_get_image($tab['image-hover'], $settings['image-size']);
					$ret .= '</div>';
				}

				$ret .= '<div class="gdlr-core-hover-box-content-wrap" ' . gdlr_core_esc_style($content_atts) . ' >';
				if( $settings['image-position'] == 'inside' && !empty($tab['image']) ){
					$ret .= '<div class="gdlr-core-hover-box-thumbnail-inside gdlr-core-media-image ';
					$ret .= (empty($tab['image-hover'])? '': 'gdlr-core-hover-thumbnail ');
					$ret .= '" ' . gdlr_core_esc_style(array(
						'width' => empty($settings['image-width'])? '': $settings['image-width']
					)) . ' >';
					$ret .= gdlr_core_get_image($tab['image'], $settings['image-size']);
					$ret .= empty($tab['image-hover'])? '': gdlr_core_get_image($tab['image-hover'], $settings['image-size']);
					$ret .= '</div>';
				}
				if( !empty($tab['title']) ){
					$ret .= '<h3 class="gdlr-core-hover-box-title gdlr-core-skin-title" ' . gdlr_core_esc_style(array(
						'font-size' => empty($settings['hover-box-title-size'])? '': $settings['hover-box-title-size'],
						'font-weight' => empty($settings['hover-box-title-font-weight'])? '': $settings['hover-box-title-font-weight'],
						'text-transform' => empty($settings['hover-box-title-text-transform'])? '': $settings['hover-box-title-text-transform'],
						'letter-spacing' => empty($settings['hover-box-title-letter-spacing'])? '': $settings['hover-box-title-letter-spacing'],
						'color' => (empty($settings['hover-box-title-color']))? '': $settings['hover-box-title-color'],
						'margin-bottom' => empty($settings['title-bottom-margin'])? '': $settings['title-bottom-margin']
					)) . ' >' . gdlr_core_text_filter($tab['title']) . '</h3>';
				}

				if( !empty($tab['content']) ){
					$ret .= '<div class="gdlr-core-hover-box-content gdlr-core-skin-content" ' . gdlr_core_esc_style(array(
						'font-size' => empty($settings['content-size']) ? '': $settings['content-size'],
						'font-weight' => empty($settings['content-font-weight']) ? '': $settings['content-font-weight'],
						'color' => empty($settings['hover-box-content-color'])? '': $settings['hover-box-content-color'],
						'margin-bottom' => empty($settings['content-bottom-margin'])? '': $settings['content-bottom-margin']
					)) . ' >' . gdlr_core_content_filter($tab['content']) . '</div>';
				}

				if( !empty($settings['link-type']) && $settings['link-type'] == 'learn-more' ){
					if( !empty($tab['link']) ){
						$ret .= '<a class="gdlr-core-hover-box-text-link" href="' . esc_url($tab['link']) .'" ' . gdlr_core_esc_style(array(
							'color' => empty($settings['hover-box-text-link-color'])? '': $settings['hover-box-text-link-color']
						)) . ' >' . esc_html__('Learn More', 'goodlayers-core') . '<i class="fa fa-long-arrow-right" ></i></a>';
					}
				}
				$ret .= '</div>'; // gdlr-core-hover-box-content

				if( empty($settings['link-type']) || $settings['link-type'] == 'box' ){
					if( !empty($tab['link']) ){
						$ret .= '<a class="gdlr-core-hover-box-link" href="' . esc_url($tab['link']) . '" ></a>';
					}
				}
				
				$ret .= '</div>'; // gdlr-core-hover-box

				return $ret;
			}			
			
		} // gdlr_core_pb_element_hover_box
	} // class_exists	