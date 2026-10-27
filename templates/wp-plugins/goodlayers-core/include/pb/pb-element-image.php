<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('image', 'gdlr_core_pb_element_image'); 
	
	if( !class_exists('gdlr_core_pb_element_image') ){
		class gdlr_core_pb_element_image{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'icon_image',
					'title' => esc_html__('Image', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;

				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'image' => array(
								'title' => esc_html__('Upload Image', 'goodlayers-core'),
								'type' => 'upload'
							),
							'thumbnail-size' => array(
								'title' => esc_html__('Thumbnail Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'thumbnail-size'
							),
							'z-index' => array(
								'title' => esc_html__('z-index', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Fill the number to position image in front of another item when it overlaps.', 'goodlayers-core') . 
									' ' . esc_html__('Only applied to front end of the site.', 'goodlayers-core')
							),
							'link-to' => array(
								'title' => esc_html__('Link To', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'lb-full-image' => esc_html__('Lightbox with full image', 'goodlayers-core'),
									'lb-custom-image' => esc_html__('Lightbox with custom image', 'goodlayers-core'),
									'lb-video' => esc_html__('Video Lightbox', 'goodlayers-core'),
									'page-url' => esc_html__('Specific Page', 'goodlayers-core'),
									'custom-url' => esc_html__('Custom Url', 'goodlayers-core'),
									'none' => esc_html__('None', 'goodlayers-core')
								),
								'default' => 'lb-full-image'
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
							'page-id' => array(
								'title' => esc_html__('Page Id', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => gdlr_core_get_post_list('page'),
								'condition' => array( 'link-to' => 'page-url' )
							),
							'custom-url' => array(
								'title' => esc_html__('Custom Url', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'link-to' => 'custom-url' )
							),
							'custom-link-target' => array(
								'title' => esc_html__('Custom Link Target', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'_self' => esc_html__('Current Screen', 'goodlayers-core'),
									'_blank' => esc_html__('New Window', 'goodlayers-core'),
								),
								'condition' => array( 'link-to' => 'custom-url' )
							),
							'overlay-icon-type' => array(
								'title' => esc_html__('Overlay Icon', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'default' => esc_html__('Default', 'goodlayers-core'),
									'custom' => esc_html__('Custom Icon', 'goodlayers-core'),
									'custom-image' => esc_html__('Custom Image', 'goodlayers-core'),
									'none' => esc_html__('None ( Background Overlay )', 'goodlayers-core'),
									'no-overlay' => esc_html__('No Overlay', 'goodlayers-core'),
								)
							),
							'overlay-icon-style' => array(
								'title' => esc_html__('Overlay Icon Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'on-hover' => esc_html__('On Hover', 'goodlayers-core'),
									'always-show-2' => esc_html__('Always Show', 'goodlayers-core'),
									'always-show' => esc_html__('Always Show ( With Round Icon )', 'goodlayers-core'),
								),
								'condition' => array( 'overlay-icon-type' => array('default', 'custom', 'custom-image') ),
							),
							'overlay-icon' => array(
								'title' => esc_html__('Icon', 'goodlayers-core'),
								'type' => 'icons',
								'allow-none' => true,
								'default' => 'fa fa-android',
								'condition' => array( 'overlay-icon-type' => 'custom' ),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'overlay-image' => array(
								'title' => esc_html__('Upload Overlay Image', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'overlay-icon-type' => 'custom-image' ),
							),
							'enable-caption' => array(
								'title' => esc_html__('Enable Caption', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
						)
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'max-width' => array(
								'title' => esc_html__('Max Width', 'goodlayers-core'),
								'data-input-type' => 'pixel',
								'type' => 'text',
							),
							'alignment' => array(
								'title' => esc_html__('Alignment ( If max width option is specified )', 'goodlayers-core'),
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
							'zoom-on-hover' => array(
								'title' => esc_html__('Thumbnail Zoom on Hover', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
							),
							'enable-shadow' => array(
								'title' => esc_html__('Enable Shadow', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'background-shadow-size' => array(
								'title' => esc_html__('Background Shadow Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
								'condition' => array( 'enable-shadow' => 'enable' )
							),
							'background-shadow-color' => array(
								'title' => esc_html__('Background Shadow Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'enable-shadow' => 'enable' )
							),
							'background-shadow-opacity' => array(
								'title' => esc_html__('Background Shadow Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core'),
								'condition' => array( 'enable-shadow' => 'enable' )
							),
							'frame-style' => array(
								'title' => esc_html__('Frame Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'rectangle' => esc_html__('Rectangle', 'goodlayers-core'),
									'round' => esc_html__('Round', 'goodlayers-core'),
									'round2' => esc_html__('Round ( Custom )', 'goodlayers-core'),
									'circle' => esc_html__('Circle', 'goodlayers-core'),
								)
							),
							'border-radius' => array(
								'title' => esc_html__('Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '3px',
								'condition' => array( 'frame-style' => 'round' )
							),
							'border-radius-2' => array(
								'title' => esc_html__('Border Radius', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('t-left', 't-right', 'b-right', 'b-left'),
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' ),
								'condition' => array( 'frame-style' => 'round2' )
							),
							'border-width' => array(
								'title' => esc_html__('Border Width', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '0px'
							),
							'overlay-icon-size' => array(
								'title' => esc_html__('Overlay Icon Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'15' => esc_html__('Small', 'goodlayers-core'),
									'22' => esc_html__('Medium', 'goodlayers-core'),
									'28' => esc_html__('Large', 'goodlayers-core'),
									'custom' => esc_html__('Custom', 'goodlayers-core')
								),
								'default' => '22'
							),
							'overlay-icon-size-custom' => array(
								'title' => esc_html__('Overlay Icon Size ( px )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '22px',
								'condition' => array( 'overlay-icon-size' => 'custom' )
							),
							'overlay-icon-bg-radius' => array(
								'title' => esc_html__('Overlay Icon Bg Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							), 
							'overlay-icon-bg-padding' => array(
								'title' => esc_html__('Overlay Icon Bg Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' ),
							), 
						)
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'border-color' => array(
								'title' => esc_html__('Border Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),		
							'overlay-color' => array(
								'title' => esc_html__('Overlay Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),		
							'overlay-icon-color' => array(
								'title' => esc_html__('Overlay Icon Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),		
							'overlay-icon-background' => array(
								'title' => esc_html__('Overlay Icon Background Color ( Round Icon Style )', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'overlay-background-opacity' => array(
								'title' => esc_html__('Overlay Background Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.6',
								'description' => esc_html__('Fill the decimal number between 0.01 to 1', 'goodlayers-core')
							)
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
							'left-right-padding' => array(
								'title' => esc_html__('Enable Left/Right Padding', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable'
							),
							'margin' => array(
								'title' => esc_html__('Margin Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link' ),
							),
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom ( Item )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),
						)
					)
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings);
				
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array() ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'image' => '', 'link-to' => 'lb-full-image', 'enable-shadow' => 'disable',
						'enable-caption' => 'enable',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$extra_class  = ' gdlr-core-' . (empty($settings['alignment'])? 'center': $settings['alignment']) . '-align';
				if( !empty($settings['mobile-alignment']) ){
					$extra_class .= ' gdlr-core-mobile-' . $settings['mobile-alignment'] . '-align'; 
				}
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$extra_class .= (empty($settings['left-right-padding']) || $settings['left-right-padding'] == 'enable')? ' gdlr-core-item-pdlr': '';
				$ret  = '<div class="gdlr-core-image-item gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				$ret .= gdlr_core_esc_style(array(
					'padding-bottom'=> (!empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb)? $settings['padding-bottom']: '',
					'transform' => empty($settings['3d-content-z-pos'])? '': 'translateZ(' . $settings['3d-content-z-pos'] . ')'
				));
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				$image_wrap_class  = (empty($settings['enable-shadow']) || $settings['enable-shadow'] == 'disable')? '': ' gdlr-core-with-shadow';
				$image_wrap_class .= ' gdlr-core-image-item-style-' . (empty($settings['frame-style'])? 'rectangle': $settings['frame-style']);
				
				$image_wrap_atts = array(
					'border-width' => empty($settings['border-width'])? '0px': $settings['border-width'],
					'border-color' => empty($settings['border-color'])? '': $settings['border-color'],
					'max-width' => empty($settings['max-width'])? '': $settings['max-width']
				);
				if( !empty($settings['frame-style']) && $settings['frame-style'] == 'round' ){
					$image_wrap_atts['border-radius'] = (empty($settings['border-radius']) || $settings['border-radius'] == '3px')? '': $settings['border-radius'];	
				}else if( !empty($settings['frame-style']) && $settings['frame-style'] == 'round2' ){
					$image_wrap_class .= str_replace('round2', 'round', $image_wrap_class);
					$image_wrap_atts['border-radius'] = (empty($settings['border-radius-2']) || $settings['border-radius-2'] == array( 
						'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
					))? '': $settings['border-radius-2'];
				}
				if( !empty($settings['enable-shadow']) && $settings['enable-shadow'] == 'enable' ){
					$image_wrap_atts['background-shadow-size'] = empty($settings['background-shadow-size'])? '': $settings['background-shadow-size'];
					$image_wrap_atts['background-shadow-color'] = empty($settings['background-shadow-color'])? '': $settings['background-shadow-color'];
					$image_wrap_atts['background-shadow-opacity'] = empty($settings['background-shadow-opacity'])? '': $settings['background-shadow-opacity'];
				}
				if( !empty($settings['z-index']) ){
					$image_wrap_atts['z-index'] = $settings['z-index'];
					$image_wrap_atts['position'] = 'relative';
				}
				if( !empty($settings['margin']) && $settings['margin'] != array('top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link') ){
					$image_wrap_atts['margin'] = $settings['margin'];
				}
				if( !empty($settings['zoom-on-hover']) && $settings['zoom-on-hover'] == 'enable' ){
					$image_wrap_class .= ' gdlr-core-zoom-on-hover';
				}

				$ret .= '<div class="gdlr-core-image-item-wrap gdlr-core-media-image ' . esc_attr($image_wrap_class) . '" ' . gdlr_core_esc_style($image_wrap_atts) . ' >'; 

				$image_atts = array();

				if( !empty($settings['link-to']) && $settings['link-to'] != 'none' ){
								
					if( $settings['link-to'] == 'lb-full-image' ){
						$image_atts['lightbox'] = true;
					}else if( $settings['link-to'] == 'lb-custom-image' ){
						if( !empty($settings['custom-image']) ){
							$image_atts['lightbox'] = 'image';
							$image_atts['lightbox-image'] = $settings['custom-image'];
						}
					}else if( $settings['link-to'] == 'lb-video' ){
						if( !empty($settings['video-url']) ){
							$image_atts['lightbox'] = 'video';
							$image_atts['lightbox-video'] = $settings['video-url'];
						}
					}else if( $settings['link-to'] == 'page-url' ){
						if( !empty($settings['page-id']) ){
							$image_atts['link'] = get_permalink($settings['page-id']);
						}
					}else if( $settings['link-to'] == 'custom-url' ){
						if( !empty($settings['custom-url']) ){
							$image_atts['link'] = $settings['custom-url'];
							$image_atts['link-target'] = empty($settings['custom-link-target'])? '': $settings['custom-link-target'];
						}
					}

					if( empty($settings['overlay-icon-type']) || $settings['overlay-icon-type'] != 'no-overlay' ){
						$image_atts['image-overlay'] = true;
						$image_atts['image-overlay-radius'] = (empty($settings['border-radius']) || $settings['border-radius'] == '3px')? '': $settings['border-radius'];
						if( isset($settings['overlay-background-opacity']) ){
							$image_atts['image-overlay-background'] = empty($settings['overlay-color'])? '': array($settings['overlay-color'], $settings['overlay-background-opacity']);
						}else{
							$image_atts['image-overlay-background'] = empty($settings['overlay-color'])? '': array($settings['overlay-color'], 0.6);
						}
						if( !empty($settings['overlay-icon-size']) ){
							if( $settings['overlay-icon-size'] == 'custom' && !empty($settings['overlay-icon-size-custom']) ){
								$image_atts['image-overlay-icon-size'] = $settings['overlay-icon-size-custom'];
							}else{
								$image_atts['image-overlay-icon-size'] = 'gdlr-core-size-' . $settings['overlay-icon-size'];
							}
						}
						$image_atts['image-overlay-icon-color'] = empty($settings['overlay-icon-color'])? '': $settings['overlay-icon-color'];

						if( !empty($settings['overlay-icon-type']) && $settings['overlay-icon-type'] == 'custom' && !empty($settings['overlay-icon']) ){
							$image_atts['image-overlay-icon'] = $settings['overlay-icon'];
							$image_atts['image-overlay-icon-type'] = 'custom';
						}else if( !empty($settings['overlay-icon-type']) && $settings['overlay-icon-type'] == 'custom-image' && !empty($settings['overlay-image']) ){
							$image_atts['image-overlay-icon'] = $settings['overlay-image'];
							$image_atts['image-overlay-icon-type'] = 'custom-image';
						} else if( !empty($settings['overlay-icon-type']) && $settings['overlay-icon-type'] == 'none' ){
							$image_atts['image-overlay-icon-type'] = 'none';
						} 

						if( !empty($settings['overlay-icon-style']) && $settings['overlay-icon-style'] == 'always-show' ){
							$image_atts['image-overlay-icon-background'] = empty($settings['overlay-icon-background'])? '': $settings['overlay-icon-background'];
							$image_atts['image-overlay-class'] = 'gdlr-core-no-hover gdlr-core-transparent gdlr-core-round-icon';
						}else if( !empty($settings['overlay-icon-style']) && $settings['overlay-icon-style'] == 'always-show-2' ){
							$image_atts['image-overlay-class'] = 'gdlr-core-no-hover gdlr-core-transparent';
						}

						$image_atts['image-overlay-icon-radius'] = empty($settings['overlay-icon-bg-radius'])? '': $settings['overlay-icon-bg-radius'];
						if( !empty($settings['overlay-icon-bg-padding']) && $settings['overlay-icon-bg-padding'] != array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' ) ){
							$image_atts['image-overlay-icon-padding'] = $settings['overlay-icon-bg-padding'];
						}
						
					}
				}

				$thumbnail_size = empty($settings['thumbnail-size'])? 'full': $settings['thumbnail-size'];
				$ret .= gdlr_core_get_image($settings['image'], $thumbnail_size, $image_atts);
				$ret .= '</div>'; // gdlr-core-image-item-wrap

				if( !empty($settings['image']) && empty($settings['enable-caption']) || $settings['enable-caption'] == 'enable' ){
					$caption = gdlr_core_get_image_info($settings['image'], 'caption');

					if( !empty($caption) ){
						$ret .= '<div class="gdlr-core-image-item-caption gdlr-core-line-height" >';
						$ret .= gdlr_core_text_filter($caption);
						$ret .= '</div>';
					}
				}
				$ret .= '</div>'; // gdlr-core-image-item
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_image
	} // class_exists	