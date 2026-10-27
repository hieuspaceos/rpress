<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('tab-feature2', 'gdlr_core_pb_element_tab_feature2'); 
	
	if( !class_exists('gdlr_core_pb_element_tab_feature2') ){
		class gdlr_core_pb_element_tab_feature2{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-folder-o',
					'title' => esc_html__('Tab Feature 2', 'goodlayers-core')
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
									'title-image' => array(
										'title' => esc_html__('Title Image', 'goodlayers-core'),
										'type' => 'upload'
									),
									'title' => array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-left-image' => array(
										'title' => esc_html__('Content Left Image', 'goodlayers-core'),
										'type' => 'upload'
									),
									'content-left-lightbox-link' => array(
										'title' => esc_html__('Content Left Lightbox Video URL', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-left-lightbox-overlay-icon' => array(
										'title' => esc_html__('Content Left Lightbox Overlay Icon Img', 'goodlayers-core'),
										'type' => 'upload'
									),
									'content-left-lightbox-overlay-icon-size' => array(
										'title' => esc_html__('Content Left Lightbox Overlay Icon Img Size', 'goodlayers-core'),
										'type' => 'text',
										'data-input-type' => 'pixel'
									),
									'content-right-box-1-title' => array(
										'title' => esc_html__('Content Right Box 1 Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-right-box-1-text' => array(
										'title' => esc_html__('Content Right Box 1 Text', 'goodlayers-core'),
										'type' => 'textarea'
									),
									'content-right-box-1-hover-banner' => array(
										'title' => esc_html__('Content Right Box 1 Hover Banner', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-right-box-2-title' => array(
										'title' => esc_html__('Content Right Box 2 Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-right-box-2-text' => array(
										'title' => esc_html__('Content Right Box 2 Text', 'goodlayers-core'),
										'type' => 'textarea'
									),
									'content-right-box-2-hover-banner' => array(
										'title' => esc_html__('Content Right Box 2 Hover Banner', 'goodlayers-core'),
										'type' => 'text'
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
							'title-image-max-width' => array(
								'title' => esc_html__('Title Image Max Width', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'thumbnail-size' => array(
								'title' => esc_html__('Content Left Image Thumbnail Size', 'goodlayers-core'),
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
							'title-slide-bar-color' => array(
								'title' => esc_html__('Title Slide Bar Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-left-image-shadow-size' => array(
								'title' => esc_html__('Content Left Image Shadow Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
							),
							'content-left-image-shadow-color' => array(
								'title' => esc_html__('Content Left Image Shadow Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'content-left-image-shadow-opacity' => array(
								'title' => esc_html__('Content Left Image Shadow Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core'),
							),
							'content-left-image-radius' => array(
								'title' => esc_html__('Content Left Image Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'content-left-lightbox-icon-bg' => array(
								'title' => esc_html__('Content Left Image Overlay Icon Background', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'hover-banner-color' => array(
								'title' => esc_html__('Hover Banner Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'move-up-effect-length' => array(
								'title' => esc_html__('Content Right Box Hover Move Up Length', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'hover-shadow-size' => array(
								'title' => esc_html__('Content Right Box Hover Shadow Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
							),
							'hover-shadow-color' => array(
								'title' => esc_html__('Content Right Box Hover Shadow Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'hover-shadow-opacity' => array(
								'title' => esc_html__('Content Right Box Hover Shadow Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core'),
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
?><script id="gdlr-core-preview-tab-feature2-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-tab-feature2-<?php echo esc_attr($id); ?>').parent().gdlr_core_tab();
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
				if( !empty($settings['hover-banner-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-tab-feature2-content-hover{ ';
					$custom_style .= ' color: ' . $settings['hover-banner-color'] . '; ';
					$custom_style .= ' background: rgba(' . gdlr_core_format_datatype($settings['hover-banner-color'], 'rgba') . ', 0.15); ';
					$custom_style .= '}'; 
				}
				if( !empty($settings['content-left-lightbox-icon-bg']) ){
					$custom_style .= '#custom_style_id .gdlr-core-tab-feature2-content-left .gdlr-core-image-overlay-icon.fa-play{ ';
					$custom_style .= ' background: rgba(' . gdlr_core_format_datatype($settings['content-left-lightbox-icon-bg'], 'rgba') . ', 0.5); ';
					$custom_style .= '}'; 
					$custom_style .= '#custom_style_id .gdlr-core-tab-feature2-content-left .gdlr-core-image-overlay-icon.fa-play:after{ ';
					$custom_style .= ' background: ' . $settings['content-left-lightbox-icon-bg'] . ';';
					$custom_style .= '}';
				}

				// hover shadow
				$custom_style_temp = gdlr_core_esc_style(array(
					'background-shadow-size' => empty($settings['hover-shadow-size'])? '': $settings['hover-shadow-size'],
					'background-shadow-color' => empty($settings['hover-shadow-color'])? '': $settings['hover-shadow-color'],
					'background-shadow-opacity' => empty($settings['hover-shadow-opacity'])? '': $settings['hover-shadow-opacity'],
				), false);
				if( !empty($settings['move-up-effect-length']) ){
					$custom_style_temp .= 'transform: translate3d(0, -' . $settings['move-up-effect-length'] . ', 0); ';
				}
				if( !empty($custom_style_temp) ){
					$custom_style .= '#custom_style_id .gdlr-core-tab-feature2-content-item:hover{ ' . $custom_style_temp . ' }';
				}

				// print
				if( !empty($custom_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_tabf2_id; 

						if( $preview ){
							$gdlr_core_tabf2_id = empty($gdlr_core_tabf2_id)? array(): $gdlr_core_tabf2_id;

							// generate unique id so it does not get overwritten in admin area
							$rnd_tabf2_id = mt_rand(0, 99999);
							while( in_array($rnd_tabf2_id, $gdlr_core_tabf2_id) ){
								$rnd_tabf2_id = mt_rand(0, 99999);
							}
							$gdlr_core_tabf2_id[] = $rnd_tabf2_id;
							$settings['id'] = 'gdlr-core-tab-feature2-' . $rnd_tabf2_id;
						}else{
							$gdlr_core_tabf2_id = empty($gdlr_core_tabf2_id)? 1: $gdlr_core_tabf2_id + 1;
							$settings['id'] = 'gdlr-core-tab-feature2-' . $gdlr_core_tabf2_id;
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
				$ret  = '<div class="gdlr-core-tab-feature2-item gdlr-core-js gdlr-core-item-pdb ' . esc_attr($tab_item_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( !empty($settings['tabs']) ){
					$count = 0; $active = 1;
					$ret .= '<div class="gdlr-core-tab-feature2-title-item-wrap clearfix" >';
					foreach( $settings['tabs'] as $tab ){ $count++;
						$ret .= '<div class="gdlr-core-tab-feature2-title-wrap ' . ($count == $active? ' gdlr-core-active': '') . ' gdlr-core-js" ';
						$ret .= 'data-tab-id="' . esc_attr($count) . '" ';
						$ret .= '>';

						if( !empty($tab['title-image']) ){
							$ret .= '<div class="gdlr-core-tab-feature2-title-image" ';
							if( !empty($settings['title-image-max-width']) ){
								$ret .= gdlr_core_esc_style(array(
									'max-width' => $settings['title-image-max-width'],
									'margin-left' => 'auto',
									'margin-right' => 'auto',
								));
							}
							$ret .= ' >';
							$ret .= gdlr_core_get_image($tab['title-image']);
							$ret .= '</div>';
						}
						if( !empty($tab['title']) ){
							$ret .= '<h3 class="gdlr-core-tab-feature2-title" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['tab-title-text'])? '': $settings['tab-title-text']
							)) . ' >' . gdlr_core_text_filter($tab['title']) . '</h3>';
						}
						$ret .= '</div>';
					}
					$ret .= '<div class="gdlr-core-tab-feature2-bottom-slide-bar" ' . gdlr_core_esc_style(array(
						'border-color' => empty($settings['title-slide-bar-color'])? '': $settings['title-slide-bar-color']
					)) . '></div>';
					$ret .= '</div>'; 
					
					
					$count = 0;
					$ret .= '<div class="gdlr-core-tab-feature2-item-content-wrap clearfix" >';
					foreach( $settings['tabs'] as $tab ){ $count++;

						$ret .= '<div class="gdlr-core-tab-feature2-content-wrap gdlr-core-container ' . ($count == $active? ' gdlr-core-active': '') . ' gdlr-core-js" ';
						$ret .= ' data-tab-id="' . esc_attr($count) . '" >';

						if( !empty($tab['content-left-image']) ){
							$image_atts = array();
							if( !empty($tab['content-left-lightbox-link']) ){
								$image_atts = array(
									'lightbox' => 'video',
									'lightbox-video' => $tab['content-left-lightbox-link'],
									'image-overlay' => true,
									'image-overlay-class' => 'gdlr-core-no-hover gdlr-core-transparent'
								);

								if( empty($tab['content-left-lightbox-overlay-icon']) ){
									$image_atts['image-overlay-icon'] = 'fa fa-play';
									$image_atts['image-overlay-icon-type'] = 'custom';
								}else{
									$image_atts['image-overlay-icon'] = $tab['content-left-lightbox-overlay-icon'];
									$image_atts['image-overlay-icon-type'] = 'custom-image';
									if( !empty($tab['content-left-lightbox-overlay-icon-size']) ){
										$image_atts['image-overlay-icon-size'] = $tab['content-left-lightbox-overlay-icon-size'];
									}
								}
							}

							$ret .= '<div class="gdlr-core-tab-feature2-content-left gdlr-core-media-image" ' . gdlr_core_esc_style(array(
								'border-radius' => empty($settings['content-left-image-radius'])? '': $settings['content-left-image-radius'],
								'background-shadow-size' => empty($settings['content-left-image-shadow-size'])? '': $settings['content-left-image-shadow-size'],
								'background-shadow-color' => empty($settings['content-left-image-shadow-color'])? '': $settings['content-left-image-shadow-color'],
								'background-shadow-opacity' => empty($settings['content-left-image-shadow-opacity'])? '': $settings['content-left-image-shadow-opacity'],
							)) . ' >';
							$ret .= gdlr_core_get_image($tab['content-left-image'], $settings['thumbnail-size'], $image_atts);
							$ret .= '</div>'; // gdlr-core-tab-feature-content-left
						}

						$ret .= '<div class="gdlr-core-tab-feature2-content-right" >';
						if( !empty($tab['content-right-box-1-title']) || !empty($tab['content-right-box-1-text']) ){
							$ret .= '<div class="gdlr-core-tab-feature2-content-item" >';
							if( !empty($tab['content-right-box-1-title']) ){
								$ret .= '<h3 class="gdlr-core-tab-feature2-content-title" >' . gdlr_core_text_filter($tab['content-right-box-1-title']) . '</h3>';
							}
							if( !empty($tab['content-right-box-1-text']) ){
								$ret .= '<div class="gdlr-core-tab-feature2-content-text" >' . gdlr_core_text_filter($tab['content-right-box-1-text']) . '</div>';
							}
							if( !empty($tab['content-right-box-1-hover-banner']) ){
								$ret .= '<div class="gdlr-core-tab-feature2-content-hover" >' . gdlr_core_text_filter($tab['content-right-box-1-hover-banner']) . '</div>';
							}
							$ret .= '</div>';

							$ret .= '<div class="gdlr-core-tab-feature2-content-item" >';
							if( !empty($tab['content-right-box-2-title']) ){
								$ret .= '<h3 class="gdlr-core-tab-feature2-content-title" >' . gdlr_core_text_filter($tab['content-right-box-2-title']) . '</h3>';
							}
							if( !empty($tab['content-right-box-2-text']) ){
								$ret .= '<div class="gdlr-core-tab-feature2-content-text" >' . gdlr_core_text_filter($tab['content-right-box-2-text']) . '</div>';
							}
							if( !empty($tab['content-right-box-2-hover-banner']) ){
								$ret .= '<div class="gdlr-core-tab-feature2-content-hover" >' . gdlr_core_text_filter($tab['content-right-box-2-hover-banner']) . '</div>';
							}
							$ret .= '</div>';
						}
						$ret .= '</div>'; // gdlr-core-tab-feature-content-right
						
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