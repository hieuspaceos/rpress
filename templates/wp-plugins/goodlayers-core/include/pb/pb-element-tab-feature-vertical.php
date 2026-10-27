<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('tab-feature-vertical', 'gdlr_core_pb_element_tab_feature_vertical'); 
	
	if( !class_exists('gdlr_core_pb_element_tab_feature_vertical') ){
		class gdlr_core_pb_element_tab_feature_vertical{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-folder-o',
					'title' => esc_html__('Tab Feature Vertical', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'caption' => array(
								'title' => esc_html__('Caption', 'goodlayers-core'),
								'type' => 'text'
							),
							'title' => array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'type' => 'text'
							),
							'tabs' => array(
								'title' => esc_html__('Add New Tab', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'tabs',
								'wrapper-class' => 'gdlr-core-fullsize',
								'options' => array(
									'title-background' => array(
										'title' => esc_html__('Head Background', 'goodlayers-core'),
										'type' => 'upload'
									),
									'title' => array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-background' => array(
										'title' => esc_html__('Content Background', 'goodlayers-core'),
										'type' => 'upload'
									),
									'content-title-image' => array(
										'title' => esc_html__('Content Title Image', 'goodlayers-core'),
										'type' => 'upload'
									),
									'content-title-caption' => array(
										'title' => esc_html__('Content Title Caption', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-title' => array(
										'title' => esc_html__('Content Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'content' => array(
										'title' => esc_html__('Content', 'goodlayers-core'),
										'type' => 'tmce'
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
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'head-background-color' => array(
								'title' => esc_html__('Head Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'title-caption-color' => array(
								'title' => esc_html__('Title Caption Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'title-color' => array(
								'title' => esc_html__('Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'title-divider-color' => array(
								'title' => esc_html__('Divider Color', 'goodlayers-core'),
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
?><script id="gdlr-core-preview-tab-featurev-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-tab-featurev-<?php echo esc_attr($id); ?>').parent().gdlr_core_tab();
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
					$settings['id'] = 'gdlr-core-tab-featurev-' . $settings['id'];
				}

				$tab_item_class  = empty($settings['no-pdlr'])? ' gdlr-core-item-pdlr': '';
				$tab_item_class .= empty($settings['class'])? '': ' ' . $settings['class'];

				// start printing item
				$ret  = '<div class="gdlr-core-tab-featurev-item gdlr-core-js gdlr-core-item-pdb ' . esc_attr($tab_item_class) . '" ';
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
					$settings['head-background-color'] = empty($settings['head-background-color'])? '': $settings['head-background-color'];
					$ret .= '<div class="gdlr-core-tab-featurev-title-item-wrap clearfix gdlr-core-js" ';
					$ret .= 'data-sync-height="' . esc_attr($settings['id']) . '-content" '; 
					$ret .= gdlr_core_esc_style(array(
						'background' => $settings['head-background-color']
					));
					$ret .= ' >';
					if( !empty($settings['title']) || !empty($settings['caption']) ){
						$ret .= '<div class="gdlr-core-tab-featurev-title-item-title-wrap" >';
						$ret .= '<div class="gdlr-core-tab-featurev-background-switch" >';
						foreach( $settings['tabs'] as $tab ){ $count++;
							$ret .= '<div class="';
							$ret .= ($active == $count)? 'gdlr-core-active ': '';
							$ret .= '" data-tab-id="' . esc_attr($count) . '" ';
							$ret .=  gdlr_core_esc_style(array(
								'background-image' => empty($tab['title-background'])? '': $tab['title-background']
							));
							$ret .= ' ></div>';
						}
						$ret .= '</div>';
						$ret .= '<div class="gdlr-core-tab-featurev-title-item-title-overlay" ' . gdlr_core_esc_style(array(
							'gradient' => array(
								array($settings['head-background-color'], 0),
								array($settings['head-background-color'], 0),
								array($settings['head-background-color'], 1),
							)
						)) . '></div>';
						$ret .= '<div data-sync-height-offset ></div>';

						if( !empty($settings['caption']) ){
							$ret .= '<div class="gdlr-core-tab-featurev-title-item-caption" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['title-caption-color'])? '': $settings['title-caption-color']
							)) . ' >' . gdlr_core_text_filter($settings['caption']) . '</div>';
						}
						if( !empty($settings['title']) ){
							$ret .= '<h3 class="gdlr-core-tab-featurev-title-item-title" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['title-color'])? '': $settings['title-color']
							)) . ' >' . gdlr_core_text_filter($settings['title']) . '</h3>';
						}
						$ret .= '<div class="gdlr-core-tab-featurev-title-divider" ' . gdlr_core_esc_style(array(
								'border-color' => empty($settings['title-divider-color'])? '': $settings['title-divider-color']
							)) . ' ></div>';
						$ret .= '</div>';
					}

					$count = 0;
					$ret .= '<ul class="gdlr-core-tab-featurev-title-wrap" >';
					foreach( $settings['tabs'] as $tab ){ $count++;
						$ret .= '<li class="gdlr-core-tab-featurev-title ';
						$ret .= ($active == $count)? 'gdlr-core-active ': '';
						$ret .= '" ';
						$ret .= ' data-tab-id="' . esc_attr($count) . '" ';
						$ret .= gdlr_core_esc_style(array(
							'color' => empty($settings['tab-title-text'])? '': $settings['tab-title-text']
						)) . ' ><i class="fa fa-angle-right" ></i>' . gdlr_core_text_filter($tab['title']) . '</li>';
					}
					$ret .= '</ul>'; // gdlr-core-tab-featurev-title-wrap
					$ret .= '</div>'; // gdlr-core-tab-featurev-title-item-wrap
					
					$count = 0;
					$ret .= '<div class="gdlr-core-tab-featurev-item-content-wrap clearfix" ' . gdlr_core_esc_style(array(
						'background' => empty($settings['content-background'])? '': $settings['content-background']
					)) . '>';
					foreach( $settings['tabs'] as $tab ){ $count++;
						$ret .= '<div class="gdlr-core-tab-featurev-content-wrap gdlr-core-js ' . ($count == $active? ' gdlr-core-active': '') . '" ';
						$ret .= ' data-tab-id="' . esc_attr($count) . '" ';
						$ret .= ' data-sync-height="' . esc_attr($settings['id']) . '-content" '; 
						$ret .= gdlr_core_esc_style(array(
							'background-image' => empty($tab['content-background'])? '': $tab['content-background']
						));
						$ret .= ' >';
						$ret .= '<div class="gdlr-core-tab-featurev-content-title-wrap" >';
						if( !empty($tab['content-title-image']) ){
							$ret .= '<div class="gldr-core-tab-featurev-content-title-image gdlr-core-media-image" >' . gdlr_core_get_image($tab['content-title-image']) . '</div>';
						}
						if( !empty($tab['content-title-caption']) ){
							$ret .= '<div class="gdlr-core-tab-featurev-content-title-caption" >' . gdlr_core_text_filter($tab['content-title-caption']) . '</div>';
						}
						if( !empty($tab['content-title']) ){
							$ret .= '<h3 class="gdlr-core-tab-featurev-content-title">' . gdlr_core_text_filter($tab['content-title']) . '</h3>';
						}
						$ret .= '</div>';

						$ret .= '<div class="gdlr-core-tab-featurev-content" >';
						$ret .= gdlr_core_content_filter($tab['content']);
						$ret .= '</div>';
						$ret .= '</div>'; // gdlr-core-tab-feature-content-wrap
					}
					$ret .= '</div>'; // gdlr-core-tab-item-tab
				}
				$ret .= '</div>'; // gdlr-core-tab-item
				
				return $ret;
			}			
			
		} // gdlr_core_pb_element_tab_feature_vertical
	} // class_exists