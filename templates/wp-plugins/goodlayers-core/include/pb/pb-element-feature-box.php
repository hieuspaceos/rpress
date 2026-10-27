<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('feature-box', 'gdlr_core_pb_element_feature_box'); 
	
	if( !class_exists('gdlr_core_pb_element_feature_box') ){
		class gdlr_core_pb_element_feature_box{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'icon_pencil-edit_alt',
					'title' => esc_html__('Featured Box', 'goodlayers-core')
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
									'icon' => esc_html__('Icon', 'goodlayers-core'),
									'image' => esc_html__('Image', 'goodlayers-core'),
								)
							),
							'icon' => array(
								'title' => esc_html__('Icon', 'goodlayers-core'),
								'type' => 'icons',
								'allow-none' => true,
								'default' => 'fa fa-android',
								'condition' => array( 'media-type' => 'icon' ),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'image' => array(
								'title' => esc_html__('Upload', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'media-type' => 'image' )
							),
							'title' => array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Feature Box Item Title', 'goodlayers-core'),
							),
							'caption' => array(
								'title' => esc_html__('Caption', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Feature Box Item Caption', 'goodlayers-core'),
							),
							'content' => array(
								'title' => esc_html__('Content', 'goodlayers-core'),
								'type' => 'tinymce',
								'default' => esc_html__('Feature Box item sample content', 'goodlayers-core'),
								'wrapper-class' => 'gdlr-core-fullsize'
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
							'link-url' => array(
								'title' => esc_html__('Item Link URL', 'goodlayers-core'),
								'type' => 'text',
								'default' => '#',
								'condition' => array( 'link-to' => 'custom-url' )
 							), 
							'link-target' => array(
								'title' => esc_html__('Item Link Target', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'_self' => esc_html__('Current Screen', 'goodlayers-core'),
									'_blank' => esc_html__('New Window', 'goodlayers-core'),
								),
								'default' => '_self',
								'condition' => array( 'link-to' => 'custom-url' )
							),	
							'text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center',
							),											
						)
					), // general
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'icon-size' => array(
								'title' => esc_html__('Icon Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '35px'
							),
							'title-size' => array(
								'title' => esc_html__('Title Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '22px'
							),
							'title-font-style' => array(
								'title' => esc_html__('Title Font Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'normal' => esc_html__('Normal', 'goodlayers-core'),
									'italic' => esc_html__('Italic', 'goodlayers-core'),
								),
								'default' => 'normal'
							),
							'title-font-weight' => array(
								'title' => esc_html__('Title Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'default' => '700',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'caption-size' => array(
								'title' => esc_html__('Caption Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '15px'
							),
							'content-size' => array(
								'title' => esc_html__('Content Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '15px'
							),
						)
					), // typography
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
							'caption-color' => array(
								'title' => esc_html__('Caption Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-color' => array(
								'title' => esc_html__('Content Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'background-color' => array(
								'title' => esc_html__('Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							), 
							'border-color' => array(
								'title' => esc_html__('Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
						)
					), //Color
					'background' => array(
						'title' => esc_html__('Background Style', 'goodlayers-core'),
						'options' => array(
							'sync-height' => array(
								'title' => esc_html__('Sync Height', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Use to sync the height among an items with the same keyword. The height will be fixed so be cautious not to use it on item with dynamic height.', 'goodlayers-core'),
							),
							'centering-sync-height-content' => array(
								'title' => esc_html__('Positioning Content In The Middle Of This Item ( When sync hieght is being used )', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'background-image' => array(
								'title' => esc_html__('Background Image', 'goodlayers-core'),
								'type' => 'upload'
							),
							'background-opacity' => array(
								'title' => esc_html__('Background Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.62',
								'description' => esc_html__('Fill the decimal number between 0.01 to 1', 'goodlayers-core')
							),
							'border-type' => array(
								'title' => esc_html__('Border Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'outer' => esc_html__('Outer Border', 'goodlayers-core'),
									'inner' => esc_html__('Inner border', 'goodlayers-core'),
								),
								'default' => 'outer'
							),
							'border-width' => array(
								'title' => esc_html__('Border Width', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'5px', 'right'=>'5px', 'bottom'=>'5px', 'left'=>'5px', 'settings'=>'link' ),
								'condition' => array( 'border-type' => array('outer', 'inner') )
							),
							'border-radius' => array(
								'title' => esc_html__('Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'default' => '3px',
								'data-input-type' => 'pixel',
								'condition' => array( 'border-type' => array('outer', 'inner') )
							),
							'pre-border-space' => array(
								'title' => esc_html__('Spaces Before Border', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'20px', 'right'=>'20px', 'bottom'=>'20px', 'left'=>'20px', 'settings'=>'link' ),
								'condition' => array( 'border-type' => 'inner' )
							),
						)
					), // background
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'content-padding' => array(
								'title' => esc_html__('Content Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top' => '50px', 'right' => '40px', 'bottom' => '40px', 'left' => '40px', 'settings' => 'unlink' )
							),
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),
						)
					), // Spacing			
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings);
				$id = mt_rand(0, 9999);
				
				if( !empty($settings['sync-height']) ){
					ob_start();
?><script id="gdlr-core-preview-feature-box-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	var feature_box_script = jQuery('#gdlr-core-preview-feature-box-<?php echo esc_attr($id); ?>');
	new gdlr_core_sync_height(feature_box_script.closest('.gdlr-core-page-builder-body'));
});
</script><?php	
					$content .= ob_get_contents();
					ob_end_clean();
				}
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array() ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'icon' => 'fa fa-android', 
						'title' => esc_html__('Feature Box Item Title', 'goodlayers-core'),
						'caption' => esc_html__('Feature Box Item Caption', 'goodlayers-core'),
						'content' => esc_html__('Feature Box item sample content', 'goodlayers-core'),
						'text-align' => 'center', 'link-url' => '#', 'link-target' => '_self',

						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// default value
				$settings['icon-size'] = (empty($settings['icon-size']) || $settings['icon-size'] == '35px')? '': $settings['icon-size'];
				$settings['title-size'] = (empty($settings['title-size']) || $settings['title-size'] == '22px')? '': $settings['title-size'];
				$settings['title-font-weight'] = (empty($settings['title-font-weight']) || $settings['title-font-weight'] == '700')? '': $settings['title-font-weight'];
				$settings['caption-size'] = (empty($settings['caption-size']) || $settings['caption-size'] == '15px')? '': $settings['caption-size'];
				$settings['content-size'] = (empty($settings['content-size']) || $settings['content-size'] == '15px')? '': $settings['content-size'];				
				$settings['border-type'] = empty($settings['border-type'])? 'outer': $settings['border-type'];
				$settings['content-padding'] =  (empty($settings['content-padding']) || $settings['content-padding'] == array(
						'top' => '50px', 'right' => '40px', 'bottom' => '40px', 'left' => '40px', 'settings' => 'unlink'
					))? '': $settings['content-padding'];
				$settings['border-width'] = (empty($settings['border-width']) || $settings['border-width'] == array( 
						'top'=>'5px', 'right'=>'5px', 'bottom'=>'5px', 'left'=>'5px', 'settings'=>'link' 
					))? '': $settings['border-width'];

				// start printing item
				$extra_class  = 'gdlr-core-' . (empty($settings['text-align'])? 'left': $settings['text-align']) . '-align';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$ret  = '<div class="gdlr-core-feature-box-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				$wrap_attr = array( 
					'background-color'=>empty($settings['background-color'])? '': $settings['background-color'],
					'padding'=>$settings['content-padding']
				);
				if( $settings['border-type'] == 'outer' ){
					$wrap_attr['border-width'] = $settings['border-width'];
					$wrap_attr['border-radius'] = empty($settings['border-radius'])? '': $settings['border-radius'];
					$wrap_attr['border-color'] = empty($settings['border-color'])? '': $settings['border-color'];
				}
				$ret .= '<div class="gdlr-core-feature-box gdlr-core-js gdlr-core-feature-box-type-' . esc_attr($settings['border-type']) . '" ';
				$ret .= gdlr_core_esc_style($wrap_attr) . ' ';
				if( !empty($settings['sync-height']) ){
					$ret .= ' data-sync-height="' . esc_attr($settings['sync-height']) . '" ';
					if( !empty($settings['centering-sync-height-content']) && $settings['centering-sync-height-content'] == 'enable' ){
						$ret .= ' data-sync-height-center';
					}
				}
				$ret .= ' >';
				if( !empty($settings['background-image']) ){
					$ret .= '<div class="gdlr-core-feature-box-background" ' . gdlr_core_esc_style(array(
						'background-image'=>$settings['background-image'],
						'opacity'=>(empty($settings['background-opacity']) || $settings['background-opacity']=='1')? '': $settings['background-opacity']
					)) . ' ></div>';
				}
				if( $settings['border-type'] == 'inner' ){
					$ret .= '<div class="gdlr-core-feature-box-frame" ' . gdlr_core_esc_style(array(
						'margin'=>(empty($settings['pre-border-space']) || $settings['pre-border-space'] == array( 
								'top'=>'20px', 'right'=>'20px', 'bottom'=>'20px', 'left'=>'20px', 'settings'=>'link'
							))? '': $settings['pre-border-space'],
						'border-width'=>$settings['border-width'],
						'border-radius'=>empty($settings['border-radius'])? '': $settings['border-radius'],
						'border-color'=>empty($settings['border-color'])? '': $settings['border-color']
					)) . ' ></div>';
				}
				$ret .= '<div class="gdlr-core-feature-box-content gdlr-core-sync-height-content" >';
				if( empty($settings['media-type']) || $settings['media-type'] == 'icon' ){
					if( !empty($settings['icon']) ){
						$ret .= '<i class="gdlr-core-feature-box-item-icon ' . esc_attr($settings['icon']) . '" ' . gdlr_core_esc_style(array(
							'font-size' => $settings['icon-size'],
							'color' => empty($settings['icon-color'])? '': $settings['icon-color']
						)) . ' ></i>';
					}
				}else{
					if( !empty($settings['image']) ){
						$ret .= '<div class="gdlr-core-feature-box-item-image gdlr-core-media-image" >';
						$ret .= gdlr_core_get_image($settings['image']);
						$ret .= '</div>';
					}
				}
				if( !empty($settings['title']) ){
					$ret .= '<h3 class="gdlr-core-feature-box-item-title" ' . gdlr_core_esc_style(array(
						'font-size' => $settings['title-size'],
						'font-style' => empty($settings['title-font-style'])? '': $settings['title-font-style'],
						'font-weight' => $settings['title-font-weight'],
						'color' => empty($settings['title-color'])? '': $settings['title-color']
					)) . ' >' . gdlr_core_text_filter($settings['title']) . '</h3>';
				}
				if( !empty($settings['caption']) ){
					$ret .= '<div class="gdlr-core-feature-box-item-caption gdlr-core-title-font" ' . gdlr_core_esc_style(array(
						'font-size' => $settings['caption-size'],
						'color' => empty($settings['caption-color'])? '': $settings['caption-color']
					)) . ' >' . gdlr_core_text_filter($settings['caption']) . '</div>';
				}
				if( !empty($settings['content']) ){
					$ret .= '<div class="gdlr-core-feature-box-item-content" ' . gdlr_core_esc_style(array(
						'font-size' => $settings['content-size'],
						'color' => empty($settings['content-color'])? '': $settings['content-color']
					)) . '>' . gdlr_core_content_filter($settings['content']) . '</div>';
				}
				$ret .= '</div>'; // gdlr-core-feature-box-content
				
				if( empty($settings['link-to']) || $settings['link-to'] == 'custom-url' ){
					if( !empty($settings['link-url']) && !empty($settings['link-target']) ){
						$ret .= '<a class="gdlr-core-feature-box-link" href="' . esc_url($settings['link-url']) . '" target="' . esc_attr($settings['link-target']) . '" ></a>';
					}
				}else if( $settings['link-to'] == 'lb-custom-image' ){
					$image_url = '';
					$caption = '';
					if( !empty($settings['custom-image']) ){
						if( is_numeric($settings['custom-image']) ){
							$image_url = gdlr_core_get_image_url($settings['custom-image']);
							$caption = gdlr_core_get_image_info($settings['custom-image'], 'caption');;
						}else{
							$image_url = $settings['custom-image'];
						}
					}

					$ret .= '<a ' . gdlr_core_get_lightbox_atts(array(
						'class'=>'gdlr-core-feature-box-link ',
						'url'=>$image_url,
						'captin'=>$caption
					)) . ' ></a>';
				}else if( $settings['link-to'] == 'lb-video' ){
					$ret .= '<a ' . gdlr_core_get_lightbox_atts(array(
						'class'=>'gdlr-core-feature-box-link ',
						'url'=>$settings['video-url'],
						'type'=>'video'
					)) . ' ></a>';
				}

				$ret .= '</div>'; // gdlr-core-feature-box-front

				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_flipbox
	} // class_exists	