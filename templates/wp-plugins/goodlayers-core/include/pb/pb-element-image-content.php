<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('image-content', 'gdlr_core_pb_element_image_content'); 
	
	if( !class_exists('gdlr_core_pb_element_image_content') ){
		class gdlr_core_pb_element_image_content{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-inbox',
					'title' => esc_html__('Image Content', 'goodlayers-core')
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
									'icon' => array(
										'title' => esc_html__('Icon', 'goodlayers-core'),
										'type' => 'text'
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
									'slide' => esc_html__('Content Slide', 'goodlayers-core'),
									'static' => esc_html__('Static', 'goodlayers-core')
								)
							),
							'text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center'
							),
							'column' => array(
								'title' => esc_html__('Column Number', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 1=>1, 2=>2, 3=>3, 4=>4, 5=>5, 6=>6),
								'default' => 3
							),
							'thumbnail-overlay-background' => array(
								'title' => esc_html__('Thumbnail Overlay Background', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'thumbnail-overlay-opacity' => array(
								'title' => esc_html__('Thumbnail Overlay Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
							'thumbnail-overlay-hover-background' => array(
								'title' => esc_html__('Thumbnail Overlay Hover Background', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'thumbnail-overlay-hover-opacity' => array(
								'title' => esc_html__('Thumbnail Overlay Hover Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
							'background-skewx' => array(
								'title' => esc_html__('Background Skew X', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'description' => esc_html__('Only input number here.', 'goodlayers-core')
							)	
						)
					),
					'typography' => array(
						'title' => esc_html__('Typograhy', 'goodlayers-core'),
						'options' => array(
							'icon-size' => array(
								'title' => esc_html__('Icon Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'title-size' => array(
								'title' => esc_html__('Title Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
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
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => 'none'
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
						)
					),				
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'icon-color' => array(
								'title' => esc_html__('Icon Color', 'goodlayers-core'),
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
							'link-color' => array(
								'title' => esc_html__('link Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Border/Spacing', 'goodlayers-core'),
						'options' => array(
							'item-height' => array(
								'title' => esc_html__('Item Height', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '300px'
							),
							'content-left-space' => array(
								'title' => esc_html__('Content Left Space', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'content-right-space' => array(
								'title' => esc_html__('Content Right Space', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'item-side-margin' => array(
								'title' => esc_html__('Item Left/Right Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom ( Wrapper )', 'goodlayers-core'),
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
?><script id="gdlr-core-preview-image-content-<?php echo esc_attr($id); ?>" >
if( document.readyState == 'complete' ){
	jQuery(document).ready(function(){
		jQuery('#gdlr-core-preview-image-content-<?php echo esc_attr($id); ?>').parent().gdlr_core_flexslider();
	});
}else{
	jQuery(window).on('load', function(){
		jQuery('#gdlr-core-preview-image-content-<?php echo esc_attr($id); ?>').parent().gdlr_core_flexslider();
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
				if( !empty($settings['thumbnail-overlay-hover-background']) ){
					$extra_style .= '#custom_style_id .gdlr-core-image-content:hover .gdlr-core-image-content-thumbnail-overlay{ background: ' . $settings['thumbnail-overlay-hover-background'] . ' !important; }';
				}
				if( !empty($settings['thumbnail-overlay-hover-opacity']) ){
					$extra_style .= '#custom_style_id .gdlr-core-image-content:hover .gdlr-core-image-content-thumbnail-overlay{ opacity: ' . $settings['thumbnail-overlay-hover-opacity'] . ' !important; }';
				}

				if( !empty($extra_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_image_content_id; 

						if( $preview ){
							$gdlr_core_image_content_id = empty($gdlr_core_image_content_id)? array(): $gdlr_core_image_content_id;

							// generate unique id so it does not get overwritten in admin area
							$rnd_id = mt_rand(0, 99999);
							while( in_array($rnd_id, $gdlr_core_image_content_id) ){
								$rnd_id = mt_rand(0, 99999);
							}
							$gdlr_core_image_content_id[] = $rnd_id;

							$settings['id'] = 'gdlr-core-image-content-id-' . $rnd_id;
						}else{
							$gdlr_core_image_content_id = empty($gdlr_core_image_content_id)? 1: $gdlr_core_image_content_id + 1;
							$settings['id'] = 'gdlr-core-image-content-id-' . $gdlr_core_image_content_id;
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
				$settings['style'] = empty($settings['style'])? 'slide': $settings['style'];
				$settings['column'] = empty($settings['column'])? '3': $settings['column'];
				$settings['text-align'] = empty($settings['text-align'])? 'left': $settings['text-align'];
				$settings['item-height'] = empty($settings['item-height'])? '300px': $settings['item-height'];

				// start printing item
				$extra_class  = empty($settings['class'])? '': ' ' . $settings['class'];
				$extra_class .= ' gdlr-core-style-' . $settings['style'];

				$ret  = $extra_style;
				$ret .= '<div class="gdlr-core-image-content-item gdlr-core-item-pdlr gdlr-core-item-pdb clearfix ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				$ret .= '<div class="gdlr-core-item-rvpdlr" ' . gdlr_core_esc_style(array(
					'margin-left' => empty($settings['item-side-margin'])? '': (floatval($settings['item-side-margin']) * -1) . 'px',
					'margin-right' => empty($settings['item-side-margin'])? '': (floatval($settings['item-side-margin']) * -1) . 'px'
				)) . ' >';
				if( !empty($settings['tabs']) ){
					$t_column_count = 0;
					$t_column = 60 / intval($settings['column']);
					foreach( $settings['tabs'] as $tab ){
						$column_class  = ' gdlr-core-column-' . $t_column;
						$column_class .= ($t_column_count % 60 == 0)? ' gdlr-core-column-first': '';

						$ret .= '<div class="gdlr-core-item-list gdlr-core-item-pdlr gdlr-core-item-mgb ' . esc_attr($column_class) . '" ' . gdlr_core_esc_style(array(
							'padding-left' => empty($settings['item-side-margin'])? '': $settings['item-side-margin'],
							'padding-right' => empty($settings['item-side-margin'])? '': $settings['item-side-margin'],
						)) . ' >';
						$ret .= self::get_tab_item($tab, $settings);
						$ret .= '</div>';

						$t_column_count += $t_column;
					}
				}
				$ret .= '</div>';
				$ret .= '</div>'; // gdlr-core-image-content-item
				
				return $ret;
			}

			static function get_tab_item( $tab = array(), $settings = array() ){ 

				$ret  = '<div class="gdlr-core-image-content gdlr-core-' . esc_attr($settings['text-align']) . '-align clearfix" ' . gdlr_core_esc_style(array(
					'height' => ($settings['item-height'] == '300px')? '': $settings['item-height'],
					'skewx' => empty($settings['background-skewx'])? '': $settings['background-skewx']
				)) . ' >';
				if( !empty($tab['background']) ){
					$pos = '';
					if( !empty($settings['background-skewx']) ){
						$tan = abs(tan(deg2rad(intval($settings['background-skewx']))));
						if( $tan > 0 ){
							$pos = (-1 * intval($settings['item-height']) * $tan / 2) . 'px';
						}
					}

					$ret .= '<div class="gdlr-core-image-content-thumbnail" ' . gdlr_core_esc_style(array(
						'skewx' => empty($settings['background-skewx'])? '': intval($settings['background-skewx']) * -1,
						'background-image' => $tab['background'],
						'left' => $pos,
						'right' => $pos
					)) . ' >';
					if( !empty($tab['link']) ){
						$ret .= '<a class="gdlr-core-image-content-link" href="' . esc_url($tab['link']) .'" ' . gdlr_core_esc_style(array(
							'color' => empty($settings['link-color'])? '': $settings['link-color']
						)) . ' >';
					}
					$ret .= '<span class="gdlr-core-image-content-thumbnail-overlay" ' . gdlr_core_esc_style(array(
						'background' => empty($settings['thumbnail-overlay-background'])? '': $settings['thumbnail-overlay-background'],
						'opacity' => empty($settings['thumbnail-overlay-opacity'])? '': $settings['thumbnail-overlay-opacity'],
					)) . ' ></span>';
					if( !empty($tab['link']) ){
						$ret .= '</a>';
					}
					$ret .= '</div>';
				}
				$ret .= '<div class="gdlr-core-image-content-overlay ' . (empty($tab['link'])? '': 'gdlr-core-with-link') . '" ' . gdlr_core_esc_style(array(
					'left' => empty($settings['content-left-space'])? '': $settings['content-left-space'],
					'right' => empty($settings['content-right-space'])? '': $settings['content-right-space'],
				)) . ' >';
				$ret .= '<div class="gdlr-core-image-content-overlay-inner" ' . gdlr_core_esc_style(array(
					'skewx' => empty($settings['background-skewx'])? '': intval($settings['background-skewx']) * -1,
				)) . ' >';
				if( !empty($tab['icon']) ){
					$ret .= '<i class="gdlr-core-image-content-icon ' . esc_attr($tab['icon']) . '" ' . gdlr_core_esc_style(array(
						'font-size' => empty($settings['icon-size'])? '': $settings['icon-size'],
						'color' => (empty($settings['icon-color']))? '': $settings['icon-color'],
					)) . ' ></i>';
				}
				if( !empty($tab['title']) ){
					$ret .= '<h3 class="gdlr-core-image-content-title gdlr-core-skin-title" ' . gdlr_core_esc_style(array(
						'font-size' => empty($settings['title-size'])? '': $settings['title-size'],
						'font-weight' => empty($settings['title-font-weight'])? '': $settings['title-font-weight'],
						'text-transform' => empty($settings['title-text-transform'])? '': $settings['title-text-transform'],
						'letter-spacing' => empty($settings['title-letter-spacing'])? '': $settings['title-letter-spacing'],
						'color' => (empty($settings['title-color']))? '': $settings['title-color'],
					)) . ' >' . gdlr_core_text_filter($tab['title']) . '</h3>';
				}

				$ret .= '<div class="gdlr-core-image-content-overlay-content" >';
				if( !empty($tab['content']) ){
					$ret .= '<div class="gdlr-core-image-content-text gdlr-core-skin-content" ' . gdlr_core_esc_style(array(
						'font-size' => empty($settings['content-size']) ? '': $settings['content-size'],
						'font-weight' => empty($settings['content-font-weight']) ? '': $settings['content-font-weight'],
						'color' => empty($settings['content-color'])? '': $settings['content-color'],
					)) . ' >' . gdlr_core_content_filter($tab['content']) . '</div>';
				}
				if( !empty($tab['link']) ){
					$ret .= '<a class="gdlr-core-image-content-link" href="' . esc_url($tab['link']) .'" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['link-color'])? '': $settings['link-color']
					)) . ' ><i class="gdlr-icon-oblique-arrow" ></i></a>';
				}
				$ret .= '</div>'; // gdlr-core-image-content-overlay-content-inner
				$ret .= '</div>'; // gdlr-core-image-content-overlay-content
				$ret .= '</div>'; // gdlr-core-image-content-overlay
				
				$ret .= '</div>'; // gdlr-core-image-content

				return $ret;
			}
			
		} // gdlr_core_pb_element_image_content
	} // class_exists	