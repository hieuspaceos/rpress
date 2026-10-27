<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('tab-feature', 'gdlr_core_pb_element_tab_feature'); 
	
	if( !class_exists('gdlr_core_pb_element_tab_feature') ){
		class gdlr_core_pb_element_tab_feature{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-folder-o',
					'title' => esc_html__('Tab Feature', 'goodlayers-core')
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
								'title' => esc_html__('Add New Tab ( At most 4 tab will be included )', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'tabs',
								'wrapper-class' => 'gdlr-core-fullsize',
								'options' => array(
									'title-background' => array(
										'title' => esc_html__('Title Background', 'goodlayers-core'),
										'type' => 'upload'
									),
									'title-background-active' => array(
										'title' => esc_html__('Title Background Active', 'goodlayers-core'),
										'type' => 'upload'
									),
									'title-image' => array(
										'title' => esc_html__('Title Image', 'goodlayers-core'),
										'type' => 'upload'
									),
									'title-image-max-width' => array(
										'title' => esc_html__('Title Image Max Width', 'goodlayers-core'),
										'type' => 'text',
										'data-input-type' => 'pixel'
									),
									'title' => array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'caption' => array(
										'title' => esc_html__('Caption', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-left-title' => array(
										'title' => esc_html__('Content Left Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-left-caption' => array(
										'title' => esc_html__('Content Left Caption', 'goodlayers-core'),
										'type' => 'textarea'
									),
									'content' => array(
										'title' => esc_html__('Content', 'goodlayers-core'),
										'type' => 'tmce'
									),
									'content-button-text' => array(
										'title' => esc_html__('Content Button Text', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-button-link' => array(
										'title' => esc_html__('Content Button Link', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-button-target' => array(
										'title' => esc_html__('Content Button Link Target', 'goodlayers-core'),
										'type' => 'combobox',
										'options' => array(
											'_self' => esc_html__('Current Screen', 'goodlayers-core'),
											'_blank' => esc_html__('New Window', 'goodlayers-core'),
										),
									),
									'content-button2-text' => array(
										'title' => esc_html__('Content Button 2 Text', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-button2-link' => array(
										'title' => esc_html__('Content Button 2 Link', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-button2-target' => array(
										'title' => esc_html__('Content Button 2 Link Target', 'goodlayers-core'),
										'type' => 'combobox',
										'options' => array(
											'_self' => esc_html__('Current Screen', 'goodlayers-core'),
											'_blank' => esc_html__('New Window', 'goodlayers-core'),
										),
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
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'enable-content-divider' => array(
								'title' => esc_html__('Enable Content Dividier', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable'
							),
							'button-style' => array(
								'title' => esc_html__('Button 1 Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'solid' => esc_html__('Solid Button', 'goodlayers-core'),
									'border' => esc_html__('Border Button', 'goodlayers-core'),
								),
								'default' => 'solid'
							),
							'button2-style' => array(
								'title' => esc_html__('Button 2 Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'solid' => esc_html__('Solid Button', 'goodlayers-core'),
									'border' => esc_html__('Border Button', 'goodlayers-core'),
								),
								'default' => 'border'
							)
						)
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'tab-title-background' => array(
								'title' => esc_html__('Tab Title Background', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'tab-title-background-active' => array(
								'title' => esc_html__('Tab Title Background Active', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'tab-title-text' => array(
								'title' => esc_html__('Tab Title Text', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'tab-title-caption' => array(
								'title' => esc_html__('Tab Title Caption', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-left-title-color' => array(
								'title' => esc_html__('Content Left Title', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-left-caption-color' => array(
								'title' => esc_html__('Content Left Caption', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-background' => array(
								'title' => esc_html__('Content Background', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-divider' => array(
								'title' => esc_html__('Content Divider', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'button-background' => array(
								'title' => esc_html__('Button Background', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'button-border' => array(
								'title' => esc_html__('Button Border', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'button-text' => array(
								'title' => esc_html__('Button Text', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'button2-background' => array(
								'title' => esc_html__('Button Background', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'button2-border' => array(
								'title' => esc_html__('Button Border', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'button2-text' => array(
								'title' => esc_html__('Button Text', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
						),
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
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
?><script id="gdlr-core-preview-tab-feature-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-tab-feature-<?php echo esc_attr($id); ?>').parent().gdlr_core_tab();
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

				global $gdlr_core_tab_feature_id; 
				if( empty($settings['id']) ){
					if( $preview ){
						$settings['id'] = mt_rand(0, 9999);
					}else{
						$gdlr_core_tab_feature_id = empty($gdlr_core_tab_feature_id)? 1: $gdlr_core_tab_feature_id + 1;
						$settings['id'] = $gdlr_core_tab_feature_id;
					}
					$settings['id'] = 'gdlr-core-tab-feature-' . $settings['id'];
				}

				$tab_item_class  = empty($settings['no-pdlr'])? ' gdlr-core-item-pdlr': '';
				$tab_item_class .= empty($settings['class'])? '': ' ' . $settings['class'];

				// start printing item
				$ret  = '<div class="gdlr-core-tab-feature-item gdlr-core-js gdlr-core-item-pdb ' . esc_attr($tab_item_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				if( !empty($settings['tabs']) ){
					$count = 0; $active = 1;
					$tab_size = (sizeof($settings['tabs']) > 3)? 4: sizeof($settings['tabs']);
					$ret .= '<div class="gdlr-core-tab-feature-title-item-wrap clearfix" >';
					foreach( $settings['tabs'] as $tab ){ $count++;
						if( $count > 4 ) return;

						$ret .= '<div class="gdlr-core-tab-feature-title-wrap gdlr-core-tab-feature-size-'. esc_attr($tab_size) . ($count == $active? ' gdlr-core-active': '') . ' gdlr-core-js" ';
						$ret .= 'data-tab-id="' . esc_attr($count) . '" '; 
						$ret .= 'data-sync-height="' . esc_attr($settings['id']) . '-title" '; 
						$ret .= '>';
						if( !empty($tab['title-background']) ){
							$ret .= '<div class="gdlr-core-tab-feature-title-background '; 
							$ret .= empty($tab['title-background-active'])? '': ' gdlr-core-with-active';
							$ret .= '" ' . gdlr_core_esc_style(array(
								'background-image' => $tab['title-background']
							)) . ' ></div>';
						}
						if( !empty($tab['title-background-active']) ){
							$ret .= '<div class="gdlr-core-tab-feature-title-background-active" ' . gdlr_core_esc_style(array(
								'background-image' => $tab['title-background-active']
							)) . ' ></div>';
						}
						$ret .= '<div class="gdlr-core-tab-feature-title-background-overlay" ' . gdlr_core_esc_style(array(
							'gradient' => empty($settings['tab-title-background'])? '': array(
								array($settings['tab-title-background'], 0.6), 
								array($settings['tab-title-background'], 1)
							)
						)) . ' ></div>';
						$ret .= '<div class="gdlr-core-tab-feature-title-background-overlay-active" ' . gdlr_core_esc_style(array(
							'gradient' => empty($settings['tab-title-background-active'])? '': array(
								array($settings['tab-title-background-active'], 0.6), 
								array($settings['tab-title-background-active'], 1)
							)
						)) . ' ></div>';

						if( !empty($tab['title-image']) ){
							$ret .= '<div class="gdlr-core-tab-feature-title-image" ';
							if( !empty($tab['title-image-max-width']) ){
								$ret .= gdlr_core_esc_style(array(
									'max-width' => $tab['title-image-max-width'],
									'margin-left' => 'auto',
									'margin-right' => 'auto',
								));
							}
							$ret .= ' >';
							$ret .= gdlr_core_get_image($tab['title-image']);
							$ret .= '</div>';
						}
						if( !empty($tab['title']) ){
							$ret .= '<h3 class="gdlr-core-tab-feature-title" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['tab-title-text'])? '': $settings['tab-title-text']
							)) . ' >' . gdlr_core_text_filter($tab['title']) . '</h3>';
						}
						if( !empty($tab['caption']) ){
							$ret .= '<div class="gdlr-core-tab-feature-caption" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['tab-title-caption'])? '': $settings['tab-title-caption']
							)) . ' >' . gdlr_core_text_filter($tab['caption']) . '</div>';
						}
						$ret .= '</div>';
					}
					$ret .= '</div>'; // gdlr-core-tab-item-tab-title-wrap
					
					$count = 0;
					$ret .= '<div class="gdlr-core-tab-feature-item-content-wrap clearfix" ' . gdlr_core_esc_style(array(
						'background' => empty($settings['content-background'])? '': $settings['content-background']
					)) . '>';
					foreach( $settings['tabs'] as $tab ){ $count++;
						if( $count > 4 ) return; 
						
						$ret .= '<div class="gdlr-core-tab-feature-content-wrap gdlr-core-container ' . ($count == $active? ' gdlr-core-active': '') . ' gdlr-core-js" ';
						$ret .= ' data-tab-id="' . esc_attr($count) . '" ' . gdlr_core_esc_style(array(
							'color'=>empty($settings['tab-content-color'])? '': $settings['tab-content-color']
						)) . ' ';
						$ret .= 'data-sync-height="' . esc_attr($settings['id']) . '-content" '; 
						$ret .= ' >';

						if( !empty($tab['content-left-image']) || !empty($tab['content-left-caption']) ){
							$ret .= '<div class="gdlr-core-tab-feature-content-left" >';
							if( !empty($tab['content-left-title']) ){
								$ret .= '<h3 class="gdlr-core-tab-feature-content-left-title" ' . gdlr_core_esc_style(array(
									'color' => empty($settings['content-left-title-color'])? '': $settings['content-left-title-color']
								)) . ' >' . gdlr_core_text_filter($tab['content-left-title']) . '</h3>';
							}
							if( !empty($tab['content-left-caption']) ){
								$ret .= '<div class="gdlr-core-tab-feature-content-left-caption gdlr-core-title-font" ' . gdlr_core_esc_style(array(
									'color' => empty($settings['content-left-caption-color'])? '': $settings['content-left-caption-color']
								)) . ' >' . gdlr_core_text_filter($tab['content-left-caption']) . '</div>';
							}
							$ret .= '</div>'; // gdlr-core-tab-feature-content-left
						}

						$ret .= '<div class="gdlr-core-tab-feature-content-right" >';
						if( empty($settings['enable-content-divider']) || $settings['enable-content-divider'] == 'enable' ){
							$ret .= '<div class="gdlr-core-tab-feature-content-right-divider" ' . gdlr_core_esc_style(array(
								'border-color' => empty($settings['content-divider'])? '': $settings['content-divider']
							)) . ' ></div>';
						}

						$ret .= gdlr_core_content_filter($tab['content']);

						// button
						$button_html = '';
						if( !empty($tab['content-button-text']) && !empty($tab['content-button-link']) ){
							$settings['button-style'] = empty($settings['button-style'])? 'solid': $settings['button-style'];
							$button_html .= '<a class="gdlr-core-tab-feature-button gdlr-core-tab-feature-button-style-' . esc_attr($settings['button-style']) . '" '; 
							$button_html .= gdlr_core_esc_style(array(
								'color' => empty($settings['button-text'])? '': $settings['button-text'],
								'background-color' => empty($settings['button-background'])? '': $settings['button-background'],
								'border-color' => empty($settings['button-border'])? '': $settings['button-border'],
							)) . ' ';
							$button_html .= 'href="' . esc_url($tab['content-button-link']) . '" ';
							$button_html .= 'target="' . (empty($tab['content-button-target'])? '_self': $tab['content-button-target']) . '" >';
							$button_html .= gdlr_core_text_filter($tab['content-button-text']);
							$button_html .= '</a>';
						}
						if( !empty($tab['content-button2-text']) && !empty($tab['content-button2-link']) ){
							$settings['button2-style'] = empty($settings['button2-style'])? 'border': $settings['button2-style'];
							$button_html .= '<a class="gdlr-core-tab-feature-button gdlr-core-tab-feature-button-style-' . esc_attr($settings['button2-style']) . '" '; 
							$button_html .= gdlr_core_esc_style(array(
								'color' => empty($settings['button2-text'])? '': $settings['button2-text'],
								'background-color' => empty($settings['button2-background'])? '': $settings['button2-background'],
								'border-color' => empty($settings['button2-border'])? '': $settings['button2-border'],
							)) . ' ';
							$button_html .= 'href="' . esc_url($tab['content-button2-link']) . '" ';
							$button_html .= 'target="' . (empty($tab['content-button2-target'])? '_self': $tab['content-button2-target']) . '" >';
							$button_html .= gdlr_core_text_filter($tab['content-button2-text']);
							$button_html .= '</a>';
						}

						if( !empty($button_html) ){
							$ret .= '<div class="gdlr-core-tab-feature-button-wrap" >' . $button_html . '</div>';
						}
						$ret .= '</div>'; // gdlr-core-tab-feature-content-right
						
						$ret .= '</div>'; // gdlr-core-tab-feature-content-wrap
					}
					$ret .= '</div>'; // gdlr-core-tab-item-tab
				}
				$ret .= '</div>'; // gdlr-core-tab-item
				
				return $ret;
			}			
			
		} // gdlr_core_pb_element_tab_feature
	} // class_exists