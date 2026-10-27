<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('feature-content', 'gdlr_core_pb_element_feature_content'); 
	
	if( !class_exists('gdlr_core_pb_element_feature_content') ){
		class gdlr_core_pb_element_feature_content{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-align-justify',
					'title' => esc_html__('Feature Content', 'goodlayers-core')
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
								'title' => esc_html__('Add Tabs', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'tabs',
								'wrapper-class' => 'gdlr-core-fullsize',
								'options' => array(
									'caption' => array(
										'title' => esc_html__('Caption', 'goodlayers-core'),
										'type' => 'text'
									),
									'title' => array(
										'title' => esc_html__('title', 'goodlayers-core'),
										'type' => 'text'
									),
									'content' => array(
										'title' => esc_html__('Content', 'goodlayers-core'),
										'type' => 'textarea'
									),
									'learn-more-text' => array(
										'title' => esc_html__('Learn More Text', 'goodlayers-core'),
										'type' => 'text',
										'default' => esc_html__('Learn More', 'goodlayers-core')
									),
									'learn-more-link' => array(
										'title' => esc_html__('Learn More Link', 'goodlayers-core'),
										'type' => 'text'
									),
									'background' => array(
										'title' => esc_html__('Background', 'goodlayers-core'),
										'type' => 'upload'
									),
									'margin-top' => array(
										'title' => esc_html__('Margin Top', 'goodlayers-core'),
										'type' => 'text',
										'data-input-type' => 'pixel'
									)
								),
								'default' => array(
									array(
										'caption' => '01',
										'title' => esc_html__('Sameple Name', 'goodlayers-core'),
										'image' => '',
									),
									array(
										'caption' => '02',
										'title' => esc_html__('Sameple Name', 'goodlayers-core'),
										'image' => '',
									),
									array(
										'caption' => '03',
										'title' => esc_html__('Sameple Name', 'goodlayers-core'),
										'image' => '',
									),
								)
							),
						),
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'text-align' => array(
								'title' => esc_html__('Text Alignment', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center',
							),
							'column' => array(
								'title' => esc_html__('Column Number', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 1=>1, 2=>2, 3=>3, 4=>4, 5=>5, 6=>6),
								'default' => 3
							),
							'title-style' => array(
								'title' => esc_html__('Title Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('With Border', 'goodlayers-core'),
									'no-border' => esc_html__('no Border', 'goodlayers-core'),
								)
							),
							'learn-more-style' => array(
								'title' => esc_html__('Learn More Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('Text', 'goodlayers-core'),
									'button' => esc_html__('Button', 'goodlayers-core'),
									'border-button' => esc_html__('Border Button', 'goodlayers-core'),
								)
							),
							'thumbnail-size' => array(
								'title' => esc_html__('Thumbnail Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'thumbnail-size',
								'default' => 'thumbnail',
							),
							'show-content-on-hover' => array(
								'title' => esc_html__('Show Content On Hover', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'carousel' => array(
								'title' => esc_html__('Enable Carousel', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'with-space' => array(
								'title' => esc_html__('With Space', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array('carousel' => 'disable')
							),
							'carousel-item-margin' => array(
								'title' => esc_html__('Carousel Item Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'carousel' => 'enable' )
							),
							'carousel-overflow' => array(
								'title' => esc_html__('Carousel Overflow', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('Hidden', 'goodlayers-core'),
									'visible' => esc_html__('Visible', 'goodlayers-core')
								),
								'condition' => array( 'carousel' => 'enable' )
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
							'carousel-navigation-icon-margin' => (function_exists('gdlr_core_get_flexslider_navigation_icon_margin')? gdlr_core_get_flexslider_navigation_icon_margin(): array()),
							'carousel-bullet-style' => array(
								'title' => esc_html__('Carousel Bullet Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => (function_exists('gdlr_core_get_flexslider_bullet_itypes')? gdlr_core_get_flexslider_bullet_itypes(): array()),
								'condition' => array( 'carousel' => 'enable', 'carousel-navigation' => array('bullet','both') ),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
						)
					),
					'typography' => array(
						'title' => esc_html__('Typograhy', 'goodlayers-core'),
						'options' => array(
							'caption-size' => array(
								'title' => esc_html__('Caption Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'caption-font-weight' => array(
								'title' => esc_html__('Caption Font Weight', 'goodlayers-core'),
								'type' => 'text',
							),
							'caption-letter-spacing' => array(
								'title' => esc_html__('Caption Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'caption-text-transform' => array(
								'title' => esc_html__('Caption Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'data-type' => 'text',
								'options' => array(
									'' => esc_html__('Default', 'goodlayers-core'),
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => 'uppercase'
							),
							'title-size' => array(
								'title' => esc_html__('Title Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'title-font-weight' => array(
								'title' => esc_html__('Title Font Weight', 'goodlayers-core'),
								'type' => 'text',
							),
							'title-letter-spacing' => array(
								'title' => esc_html__('Title Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'title-text-transform' => array(
								'title' => esc_html__('Title Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'data-type' => 'text',
								'options' => array(
									'' => esc_html__('Default', 'goodlayers-core'),
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => 'uppercase'
							),
							'content-size' => array(
								'title' => esc_html__('Content Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'content-font-weight' => array(
								'title' => esc_html__('Content Font Weight', 'goodlayers-core'),
								'type' => 'text',
							),
							'content-letter-spacing' => array(
								'title' => esc_html__('Content Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'content-text-transform' => array(
								'title' => esc_html__('Content Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'data-type' => 'text',
								'options' => array(
									'' => esc_html__('Default', 'goodlayes-core'),
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								)
							),
							'learn-more-size' => array(
								'title' => esc_html__('Learn More Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'learn-more-font-weight' => array(
								'title' => esc_html__('Learn More Font Weight', 'goodlayers-core'),
								'type' => 'text',
							),
							'learn-more-letter-spacing' => array(
								'title' => esc_html__('Learn More Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'learn-more-text-transform' => array(
								'title' => esc_html__('Learn More Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'data-type' => 'text',
								'options' => array(
									'' => esc_html__('Default', 'goodlayes-core'),
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								)
							),
						)
					),				
					'frame' => array(
						'title' => esc_html__('Frame/Shadow', 'goodlayers-core'),
						'options' => array(
							'content-padding' => array(
								'title' => esc_html__('Content', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' )
							),
							'frame-border-size' => array(
								'title' => esc_html__('Border Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' )
							),
							'frame-border-color' => array(
								'title' => esc_html__('Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'descripiton' => esc_html__('Only effects the "Column With Frame" style', 'goodlayers-core')
							),
							'frame-border-radius' => array(
								'title' => esc_html__('Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'frame-shadow-size' => array(
								'title' => esc_html__('Shadow Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
							),
							'frame-shadow-color' => array(
								'title' => esc_html__('Shadow Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'frame-shadow-opacity' => array(
								'title' => esc_html__('Shadow Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
						)
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'caption-color' => array(
								'title' => esc_html__('Caption Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'title-color' => array(
								'title' => esc_html__('Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-color' => array(
								'title' => esc_html__('Content Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'learn-more-color' => array(
								'title' => esc_html__('Learn More Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'learn-more-bg-color' => array(
								'title' => esc_html__('Learn More Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'learn-more-border-color' => array(
								'title' => esc_html__('Learn More Border Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'background-color' => array(
								'title' => esc_html__('Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'background-opacity' => array(
								'title' => esc_html__('Background Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.4',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core'),
							),
							'background-hover-opacity' => array(
								'title' => esc_html__('Background Hover Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.8',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core'),
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'carousel-bullet-top-margin' => array(
								'title' => esc_html__('Carousel Bullet Top Margin', 'goodlayers-core'),
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
?><script id="gdlr-core-preview-feature-content-<?php echo esc_attr($id); ?>" >
if( document.readyState == 'complete' ){
	jQuery(document).ready(function(){
		jQuery('#gdlr-core-preview-feature-content-<?php echo esc_attr($id); ?>').parent().gdlr_core_flexslider();
	});
}else{
	jQuery(window).load(function(){
		jQuery('#gdlr-core-preview-feature-content-<?php echo esc_attr($id); ?>').parent().gdlr_core_flexslider();
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
								'caption' => '01',
								'title' => esc_html__('Sameple Name', 'goodlayers-core'),
								'image' => '',
							),
							array(
								'caption' => '02',
								'title' => esc_html__('Sameple Name', 'goodlayers-core'),
								'image' => '',
							),
							array(
								'caption' => '03',
								'title' => esc_html__('Sameple Name', 'goodlayers-core'),
								'image' => '',
							),
						),
						'column' => 3, 'carousel' => 'disable',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// default value
				$settings['column'] = empty($settings['column'])? '3': $settings['column'];
				$settings['carousel'] = empty($settings['carousel'])? 'disable': $settings['carousel'];
				$settings['with-space'] = empty($settings['with-space'])? 'enable': $settings['with-space']; 

				// custom css
				$custom_style  = '';
				if( !empty($settings['background-hover-opacity']) ){
					$custom_style .= '#custom_style_id .gdlr-core-feature-content:hover .gdlr-core-feature-box-overlay{ opacity: ' . $settings['background-hover-opacity'] . ' !important; }';
				}
				if( !empty($settings['carousel-navigation-icon-hover-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-flexslider-custom-nav i:hover{ color: ' . $settings['carousel-navigation-icon-hover-color'] . ' !important; }';
				}
				if( !empty($custom_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_fc_id; 
						if( $preview ){
							$gdlr_core_fc_id = empty($gdlr_core_fc_id)? array(): $gdlr_core_fc_id;

							// generate unique id so it does not get overwritten in admin area
							$rnd_fc_id = mt_rand(0, 99999);
							while( in_array($rnd_fc_id, $gdlr_core_fc_id) ){
								$rnd_fc_id = mt_rand(0, 99999);
							}
							$gdlr_core_fc_id[] = $rnd_fc_id;
							$settings['id'] = 'gdlr-core-feature-content-' . $rnd_fc_id;
						}else{
							$gdlr_core_fc_id = empty($gdlr_core_fc_id)? 1: $gdlr_core_fc_id + 1;
							$settings['id'] = 'gdlr-core-feature-content-' . $gdlr_core_fc_id;
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
				$extra_class  = ($settings['carousel'] == 'enable' || $settings['with-space'] == 'disable')? ' gdlr-core-item-pdlr': '';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				
				$ret  = '<div class="gdlr-core-feature-content-item gdlr-core-item-pdb clearfix ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				// grid item
				if( $settings['carousel'] == 'disable' ){

					if( !empty($settings['tabs']) ){
						$t_column_count = 0;
						$t_column = 60 / intval($settings['column']);
						foreach( $settings['tabs'] as $tab ){
							$column_class  = ' gdlr-core-column-' . $t_column;
							$column_class .= ($t_column_count % 60 == 0)? ' gdlr-core-column-first': '';
							$column_class .= ($settings['with-space'] == 'enable')? ' gdlr-core-item-pdlr gdlr-core-item-mgb': '';

							$ret .= '<div class="' . esc_attr($column_class) . '" >';
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
						'margin' => empty($settings['carousel-item-margin'])? '': $settings['carousel-item-margin'],
						'overflow' => empty($settings['carousel-overflow'])? '': $settings['carousel-overflow'],
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
						'navigation-icon-margin' => empty($settings['carousel-navigation-icon-margin'])? '': $settings['carousel-navigation-icon-margin'],
						'navigation-left-icon' => empty($settings['carousel-navigation-left-icon'])? '': $settings['carousel-navigation-left-icon'],
						'navigation-right-icon' => empty($settings['carousel-navigation-right-icon'])? '': $settings['carousel-navigation-right-icon'],
						'bullet-style' => empty($settings['carousel-bullet-style'])? '': $settings['carousel-bullet-style'],
						'controls-top-margin' => empty($settings['carousel-bullet-top-margin'])? '': $settings['carousel-bullet-top-margin'],
						'disable-autoslide' => (empty($settings['carousel-autoslide']) || $settings['carousel-autoslide'] == 'enable')? '': true,
					);

					if( !empty($settings['tabs']) ){
						foreach( $settings['tabs'] as $tab ){
							$slides[] = self::get_tab_item($tab, $settings);
						}
					}

					$ret .= gdlr_core_get_flexslider($slides, $flex_atts);
				}

				$ret .= '</div>'; 
				$ret .= $custom_style;
				
				return $ret;
			}

			static function get_tab_item( $tab = array(), $settings = array() ){
				
				$frame_css = array(
					'margin-top' => empty($tab['margin-top'])? '': $tab['margin-top'],
					'border-width' => (empty($settings['frame-border-size']) || $settings['frame-border-size'] == array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link'))? '': $settings['frame-border-size'],
					'border-color' => empty($settings['frame-border-color'])? '': $settings['frame-border-color'],
					'border-radius' => empty($settings['frame-border-radius'])? '': $settings['frame-border-radius'],
					'background-shadow-size' => empty($settings['frame-shadow-size'])? '': $settings['frame-shadow-size'],
					'background-shadow-color' => empty($settings['frame-shadow-color'])? '': $settings['frame-shadow-color'],
					'background-shadow-opacity' => empty($settings['frame-shadow-opacity'])? '': $settings['frame-shadow-opacity'],
				);	
				$extra_class = '';
				if( !empty($frame_css['background-shadow-size']) && !empty($frame_css['background-shadow-color']) && !empty($frame_css['background-shadow-opacity']) ){
					$extra_class .= 'gdlr-core-outer-frame-element';
				}

				$extra_class .= ' gdlr-core-' . (empty($settings['text-align'])? 'left': $settings['text-align']) . '-align';

				$ret  = '<div class="gdlr-core-feature-content clearfix ' . $extra_class . '" ' . gdlr_core_esc_style($frame_css) . ' >';

				$box_class = 'gdlr-core-without-background';
				if( !empty($tab['background']) ){
					$settings['thumbnail-size'] = empty($settings['thumbnail-size'])? 'full': $settings['thumbnail-size'];
					$ret .= '<div class="gdlr-core-feature-content-thumbnail gdlr-core-media-image" >' . gdlr_core_get_image($tab['background'], $settings['thumbnail-size']) . '</div>';
					$ret .= '<div class="gdlr-core-feature-box-overlay" ' . gdlr_core_esc_style(array(
						'background' => empty($settings['background-color'])? '#000000': $settings['background-color'],
						'opacity' => empty($settings['background-opacity'])? '0.4': $settings['background-opacity'],
					)) . ' ></div>';
					$box_class = 'gdlr-core-with-background';
				}
				
				
				$ret .= '<div class="gdlr-core-feature-content-box ' . esc_attr($box_class) . '" ' . gdlr_core_esc_style(array(
					'padding' => (empty($settings['content-padding']) || $settings['content-padding'] == array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' ))? '': $settings['content-padding'],
					'background' => ($box_class == 'gdlr-core-without-background' && !empty($settings['background-color']))? $settings['background-color']: ''
				)) . ' >';
				if( !empty($tab['caption']) ){
					$ret .= '<div class="gdlr-core-feature-content-caption gdlr-core-skin-info" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['caption-color'])? '': $settings['caption-color'],
						'font-size' => empty($settings['caption-size'])? '': $settings['caption-size'],
						'font-weight' => empty($settings['caption-font-weight'])? '': $settings['caption-font-weight'],
						'font-style' => empty($settings['caption-font-style'])? '': $settings['caption-font-style'],
						'letter-spacing' => empty($settings['caption-letter-spacing'])? '': $settings['caption-letter-spacing'],
						'text-transform' => (empty($settings['caption-text-transform']))? '': $settings['caption-text-transform'],
						'line-height' => empty($settings['caption-line-height'])? '': $settings['caption-line-height'],
					)) . ' >';
					$ret .= gdlr_core_text_filter($tab['caption']);
					$ret .= '</div>';
				}

				if( !empty($tab['title']) ){
					$ret .= '<div class="gdlr-core-feature-content-title gdlr-core-title-font gdlr-core-skin-title';
					$ret .= empty($settings['title-style'])? '': ' gdlr-core-' . $settings['title-style'];
					$ret .= '" ' . gdlr_core_esc_style(array(
						'color' => (empty($settings['title-color']))? '': $settings['title-color'],
						'font-size' => (empty($settings['title-size']))? '': $settings['title-size'],
						'font-weight' => (empty($settings['title-font-weight']))? '': $settings['title-font-weight'],
						'letter-spacing' => (empty($settings['title-letter-spacing']))? '': $settings['title-letter-spacing'],
						'text-transform' => (empty($settings['title-text-transform']))? '': $settings['title-text-transform'],
					)) . ' >' . gdlr_core_text_filter($tab['title']) . '</div>';
				}

				$ret .= (empty($settings['show-content-on-hover']) || $settings['show-content-on-hover'] == 'disable')? '': '<div class="gdlr-core-feature-content-wrap" >';
				if( !empty($tab['content']) ){
					$ret .= '<div class="gdlr-core-feature-content-content gdlr-core-skin-content" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['content-color'])? '': $settings['content-color'],
						'font-size' => empty($settings['content-size'])? '': $settings['content-size'],
						'font-weight' => empty($settings['content-font-weight'])? '': $settings['content-font-weight'],
						'font-style' => empty($settings['content-font-style'])? '': $settings['content-font-style'],
						'letter-spacing' => empty($settings['content-letter-spacing'])? '': $settings['content-letter-spacing'],
						'text-transform' => (empty($settings['content-text-transform']))? '': $settings['content-text-transform'],
						'line-height' => empty($settings['content-line-height'])? '': $settings['content-line-height'],
					)) . ' >';
					$ret .= gdlr_core_content_filter($tab['content']);
					$ret .= '</div>';
				}

				if( !empty($tab['learn-more-link']) && !empty($tab['learn-more-text']) ){

					$learn_more_atts = array(
						'color' => empty($settings['learn-more-color'])? '': $settings['learn-more-color'],
						'font-size' => empty($settings['learn-more-size'])? '': $settings['learn-more-size'],
						'font-weight' => empty($settings['learn-more-font-weight'])? '': $settings['learn-more-font-weight'],
						'letter-spacing' => empty($settings['learn-more-letter-spacing'])? '': $settings['learn-more-letter-spacing'],
						'text-transform' => (empty($settings['learn-more-text-transform']))? '': $settings['learn-more-text-transform'],
					);
					$learn_more_class = 'gdlr-core-feature-content-learn-more';
					if( !empty($settings['learn-more-style'])){
						if( $settings['learn-more-style'] == 'button' ){
							$learn_more_text = gdlr_core_text_filter($tab['learn-more-text']);
							$learn_more_class .= ' gdlr-core-button';
							$learn_more_atts['background-color'] = empty($settings['learn-more-bg-color'])? '': $settings['learn-more-bg-color'];
 						}else if( $settings['learn-more-style'] == 'border-button' ){
							$learn_more_text = gdlr_core_text_filter($tab['learn-more-text']);
							$learn_more_class .= ' gdlr-core-button gdlr-core-button-with-border gdlr-core-button-transparent';
							$learn_more_atts['border-color'] = empty($settings['learn-more-border-color'])? '': $settings['learn-more-border-color'];
						}
					}else{
						$learn_more_atts['border-color'] = empty($settings['learn-more-border-color'])? '': $settings['learn-more-border-color'];
						$learn_more_text = gdlr_core_text_filter($tab['learn-more-text']) . '<i class="ion-ios-arrow-thin-right" ></i>';
					}

					$ret .= '<a class="' . esc_attr($learn_more_class) . '" href="' . esc_attr($tab['learn-more-link']) . '" ' . gdlr_core_esc_style($learn_more_atts) . ' >' . $learn_more_text . '</a>';
				}
				$ret .= (empty($settings['show-content-on-hover']) || $settings['show-content-on-hover'] == 'disable')? '': '</div>';

				$ret .= '</div>'; // gdlr-core-feature-content-box
				$ret .= '</div>'; // gdlr-core-feature-content

				return $ret; 
			}	
			
		} // gdlr_core_pb_element_feature_content
	} // class_exists	