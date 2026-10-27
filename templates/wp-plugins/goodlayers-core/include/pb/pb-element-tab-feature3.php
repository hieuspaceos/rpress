<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('tab-feature3', 'gdlr_core_pb_element_tab_feature3'); 
	
	if( !class_exists('gdlr_core_pb_element_tab_feature3') ){
		class gdlr_core_pb_element_tab_feature3{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-folder-o',
					'title' => esc_html__('Tab Feature 3', 'goodlayers-core')
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
									'title' => array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-left-title' => array(
										'title' => esc_html__('Content Left Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-left-text' => array(
										'title' => esc_html__('Content Left Text', 'goodlayers-core'),
										'type' => 'textarea'
									),
									'content-left-button-text' => array(
										'title' => esc_html__('Content Left Button Text', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-left-button-link' => array(
										'title' => esc_html__('Content Left Button Link', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-right-image' => array(
										'title' => esc_html__('Content Right Image', 'goodlayers-core'),
										'type' => 'upload'
									),
								),
								'default' => array(
									array(
										'title' => esc_html__('Tab Title', 'goodlayers-core'),
										'content' => esc_html__('Tab content area', 'goodlayers-core'),
									),
									array(
										'title' => esc_html__('Tab Title', 'goodlayers-core'),
										'content' => esc_html__('Tab content area', 'goodlayers-core'),
									),
								)
							),
						),
					),
					'settings' => array(
						'title' => esc_html__('Settings', 'goodlayers-core'),
						'options' => array(
							'thumbnail-size' => array(
								'title' => esc_html__('Content Right Image Thumbnail Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'thumbnail-size',
								'default' => 'full'
							)
						)
					),
					'color' => array(
						'title' => esc_html__('Style/Color', 'goodlayers-core'),
						'options' => array(
							'tab-title-text' => array(
								'title' => esc_html__('Tab Title Text', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'tab-title-active-text' => array(
								'title' => esc_html__('Tab Title Active Text', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'title-slide-bar-color' => array(
								'title' => esc_html__('Title Slide Bar Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),

							'content-left-title-color' => array(
								'title' => esc_html__('Content Left Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-left-text-color' => array(
								'title' => esc_html__('Content Left Text Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-left-button-color' => array(
								'title' => esc_html__('Content Left Button Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
						),
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'left-content-top-padding' => array(
								'title' => esc_html__('Left Content Top Padding', 'goodlayers-core'),
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
?><script id="gdlr-core-preview-tab-feature3-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-tab-feature3-<?php echo esc_attr($id); ?>').parent().gdlr_core_tab();
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
						'tabs' => array(
							array(
								'title' => esc_html__('Tab Title', 'goodlayers-core'),
								'content' => esc_html__('Tab content area', 'goodlayers-core'),
							),
							array(
								'title' => esc_html__('Tab Title', 'goodlayers-core'),
								'content' => esc_html__('Tab content area', 'goodlayers-core'),
							),
						),
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				$settings['thumbnail-size'] = empty($settings['thumbnail-size'])? 'full': $settings['thumbnail-size'];

				$tab_item_class  = empty($settings['no-pdlr'])? ' gdlr-core-item-pdlr': '';
				$tab_item_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$tab_item_class .= empty($settings['tabs'])? '': ' gdlr-core-size-' . sizeOf($settings['tabs']);

				// custom style
				$custom_style  = '';
				if( !empty($settings['tab-title-text']) ){
					$custom_style .= '#custom_style_id .gdlr-core-tab-feature3-title-wrap .gdlr-core-tab-feature3-title{ color: ' . $settings['tab-title-text'] . '; }';
				}
				if( !empty($settings['tab-title-active-text']) ){
					$custom_style .= '#custom_style_id .gdlr-core-tab-feature3-title-wrap.gdlr-core-active .gdlr-core-tab-feature3-title{ color: ' . $settings['tab-title-text'] . '; }';
				}

				// print
				if( !empty($custom_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_tabf3_id; 
						
						if( $preview ){
							$gdlr_core_tabf3_id = empty($gdlr_core_tabf3_id)? array(): $gdlr_core_tabf3_id;

							// generate unique id so it does not get overwritten in admin area
							$rnd_tabf3_id = mt_rand(0, 99999);
							while( in_array($rnd_tabf3_id, $gdlr_core_tabf3_id) ){
								$rnd_tabf3_id = mt_rand(0, 99999);
							}
							$gdlr_core_tabf3_id[] = $rnd_tabf3_id;
							$settings['id'] = 'gdlr-core-tab-feature3-' . $rnd_tabf3_id;
						}else{
							$gdlr_core_tabf3_id = empty($gdlr_core_tabf3_id)? 1: $gdlr_core_tabf3_id + 1;
							$settings['id'] = 'gdlr-core-tab-feature3-' . $gdlr_core_tabf3_id;
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
				$ret  = '<div class="gdlr-core-tab-feature3-item gdlr-core-js gdlr-core-item-pdb ' . esc_attr($tab_item_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( !empty($settings['tabs']) ){
					$count = 0; $active = 1;
					$ret .= '<div class="gdlr-core-tab-feature3-title-item-wrap clearfix" >';
					foreach( $settings['tabs'] as $tab ){ $count++;
						$ret .= '<div class="gdlr-core-tab-feature3-title-wrap gdlr-core-column-' . (60 / count($settings['tabs'])) . ' ';
						$ret .= ($count == $active? ' gdlr-core-active': '') . ' gdlr-core-js" ';
						$ret .= 'data-tab-id="' . esc_attr($count) . '" ';
						$ret .= '>';
						if( !empty($tab['title']) ){
							$ret .= '<h3 class="gdlr-core-tab-feature3-title" >' . gdlr_core_text_filter($tab['title']) . '</h3>';
						}
						$ret .= '</div>';
					}
					$ret .= '<div class="gdlr-core-tab-feature3-bottom-slide-bar" ><div class="gdlr-core-tab-feature3-bottom-slide-bar-border" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['title-slide-bar-color'])? '': $settings['title-slide-bar-color'],
						'border-color' => empty($settings['title-slide-bar-color'])? '': $settings['title-slide-bar-color']
					)) . ' ></div></div>';
					$ret .= '</div>'; 
					
					
					$count = 0;
					$ret .= '<div class="gdlr-core-tab-feature3-item-content-wrap clearfix" >';
					foreach( $settings['tabs'] as $tab ){ $count++;
						$ret .= '<div class="gdlr-core-tab-feature3-content-wrap gdlr-core-container ' . ($count == $active? ' gdlr-core-active': '') . ' gdlr-core-js" ';
						$ret .= ' data-tab-id="' . esc_attr($count) . '" >';

						if( !empty($tab['content-right-image']) ){
							$ret .= '<div class="gdlr-core-tab-feature3-content-right gdlr-core-media-image" >';
							$ret .= gdlr_core_get_image($tab['content-right-image'], $settings['thumbnail-size']);
							$ret .= '</div>'; // gdlr-core-tab-feature-content-right
						}

						$ret .= '<div class="gdlr-core-tab-feature3-content" ' . gdlr_core_esc_style(array(
							'padding-top' => empty($settings['left-content-top-padding'])? '': $settings['left-content-top-padding']
						)) . ' >';
						if( !empty($tab['content-left-title']) ){
							$ret .= '<div class="gdlr-core-tab-feature3-content-title" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['content-left-title-color'])? '': $settings['content-left-title-color']
							)) . ' >' . gdlr_core_text_filter($tab['content-left-title']) . '</div>';
						}
						if( !empty($tab['content-left-text']) ){
							$ret .= '<div class="gdlr-core-tab-feature3-content-text" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['content-left-text-color'])? '': $settings['content-left-text-color']
							)) . ' >' . gdlr_core_content_filter($tab['content-left-text']) . '</div>';
						}
						if( !empty($tab['content-left-button-text']) && !empty($tab['content-left-button-link']) ){
							$ret .= '<div class="gdlr-core-tab-feature3-content-button" >';
							$ret .= '<a class="gdlr-core-button gdlr-core-button-transparent gdlr-core-button-with-border" href="' . esc_attr($tab['content-left-button-link']) . '" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['content-left-button-color'])? '': $settings['content-left-button-color'],
								'border-color' => empty($settings['content-left-button-color'])? '': $settings['content-left-button-color'],
							)) . ' >' . gdlr_core_text_filter($tab['content-left-button-text']) . '</a>';
							$ret .= '</div>';
						}
						$ret .= '</div>'; // gdlr-core-tab-feature-content-left

						$ret .= '</div>'; // gdlr-core-tab-feature-content-wrap
					}
					$ret .= '</div>'; // gdlr-core-tab-item-tab */
				}
				$ret .= '</div>'; // gdlr-core-tab-item
				$ret .= $custom_style;

				return $ret;
			}			
			
		} // gdlr_core_pb_element_tab_feature2
	} // class_exists