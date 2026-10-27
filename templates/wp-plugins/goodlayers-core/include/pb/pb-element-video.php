<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('video', 'gdlr_core_pb_element_video'); 
	
	if( !class_exists('gdlr_core_pb_element_video') ){
		class gdlr_core_pb_element_video{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-video-camera',
					'title' => esc_html__('Video', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'privacy' => array(
								'title' => esc_html__('Privacy', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => gdlr_core_get_pb_privacy_options(),
								'description' => esc_html__('Use to omit the content before accepting the consent. You can create privacy settings at the "Theme option > Miscalleneous" area.', 'goodlayers-core')
							),
							'video-type' => array(
								'title' => esc_html__('Video Type', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'html5' => GDLR_CORE_URL . '/include/images/video/html5.png',
									'youtube' => GDLR_CORE_URL . '/include/images/video/youtube.png',
									'vimeo' => GDLR_CORE_URL . '/include/images/video/vimeo.png',
								),
								'default' => 'youtube',
								'wrapper-class' => 'gdlr-core-fullsize'
							),		
							'video-url' => array(
								'title' => esc_html__('Video URL', 'goodlayers-core'),
								'type' => 'text',
								'default' => 'https://www.youtube.com/watch?v=Ow2Shb_nkOw',
								'condition' => array( 'video-type' => array('youtube','vimeo') )
							),
							'video-url-mp4' => array(
								'title' => esc_html__('Background Video URL (MP4)', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'video-type' => 'html5' ),
							),
							'video-url-webm' => array(
								'title' => esc_html__('Background Video URL (WEBM)', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'video-type' => 'html5' ),
							),
							'video-url-ogg' => array(
								'title' => esc_html__('Background Video URL (ogg)', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'video-type' => 'html5' ),
							),
							'thumbnail-image' => array(
								'title' => esc_html__('Thumbnail Image', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'video-type' => 'html5' ),
							),
							'autoplay' => array(
								'title' => esc_html__('Autoplay', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'loop' => array(
								'title' => esc_html__('Loop', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'video-overlay' => 	array(
								'title' => esc_html__('Video Overlay', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'image' => esc_html__('Image', 'goodlayers-core')
								),
								'condition' => array( 'video-type' => array('youtube', 'vimeo') )
							),
							'video-overlay-image' => array(
								'title' => esc_html__('Overlay Image', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'video-type' => array('youtube', 'vimeo'), 'video-overlay' => 'image' )
							),
							'video-overlay-image-opacity' => array(
								'title' => esc_html__('Overlay Image Opacity', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'video-type' => array('youtube', 'vimeo'), 'video-overlay' => 'image' )
							),
							'video-overlay-background-color' => array(
								'title' => esc_html__('Overlay Background Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'video-type' => array('youtube', 'vimeo'), 'video-overlay' => 'image' )
							),
							'video-overlay-icon-color' => array(
								'title' => esc_html__('Overlay Icon Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'video-type' => array('youtube', 'vimeo'), 'video-overlay' => 'image' )
							)
						)
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
				$content  = self::get_content($settings);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-video-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-video-<?php echo esc_attr($id); ?>').parent().gdlr_core_content_script().gdlr_core_mejs_ajax();
});
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array() ){
				global $gdlr_core_item_pdb;
	
				// default variable
				if( empty($settings) ){
					$settings = array(
						'video-type' => 'youtube',
						'video-url' => 'https://www.youtube.com/watch?v=Ow2Shb_nkOw',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				$settings['video-type'] = empty($settings['video-type'])? 'youtube': $settings['video-type'];
				$settings['autoplay'] = (empty($settings['autoplay']) || $settings['autoplay'] == 'disable')? false: true;
				$settings['loop'] = (empty($settings['loop']) || $settings['loop'] == 'disable')? false: true;

				// start printing item
				$extra_class  = empty($settings['class'])? '': $settings['class'];
				$ret  = '<div class="gdlr-core-video-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				// privacy settings
				if( !is_admin() && !empty($settings['privacy']) ){
					$privacy_content = gdlr_core_get_pb_privacy_box($settings['privacy'], 'pb-item');
				}
				if( !empty($privacy_content) ){
					$ret .= $privacy_content;
				}else{
					$ret .= '<div class="gdlr-core-video-item-type-' . esc_attr($settings['video-type']) . '" >';
					if( $settings['video-type'] == 'html5' ){
						$video_atts = array();
						if( !empty($settings['video-url-mp4']) ){
							$video_atts['mp4'] = $settings['video-url-mp4'];
						}
						if( !empty($settings['video-url-webm']) ){
							$video_atts['webm'] = $settings['video-url-webm'];
						}
						if( !empty($settings['video-url-ogg']) ){
							$video_atts['ogg'] = $settings['video-url-ogg'];
						}
						$video_atts['autoplay'] = $settings['autoplay'];
						$video_atts['loop'] = $settings['loop'];
						if( !empty($settings['thumbnail-image']) ){
							$video_atts['poster'] = wp_get_attachment_url($settings['thumbnail-image']);
						}
						$ret .= wp_video_shortcode($video_atts);
					}else{
						if( !empty($settings['video-url']) ){
							$ret .= gdlr_core_get_video($settings['video-url'], 'full', array(
								'autoplay' => $settings['autoplay'],
								'loop' => $settings['loop'],
								'api' => '1'
							));
						}

						if( !empty($settings['video-overlay']) && $settings['video-overlay'] == 'image' ){
							$ret .= '<div class="gdlr-core-video-item-overlay" ' . gdlr_core_esc_style(array(
								'background' => empty($settings['video-overlay-background-color'])? '': $settings['video-overlay-background-color']
							)) . ' >';
							if( !empty($settings['video-overlay-image']) ){
								$ret .= '<div class="gdlr-core-video-item-overlay-image" ' . gdlr_core_esc_style(array(
									'background-image' => $settings['video-overlay-image'],
									'opacity' => empty($settings['video-overlay-image-opacity'])? '': $settings['video-overlay-image-opacity']
								)) . ' ></div>';
							}
							$ret .= '<div class="gdlr-core-video-item-overlay-icon" ><i class="fa fa-play"></i></div>';
							$ret .= '</div>';
						}
					}
					$ret .= '</div>'; // video-item-type
				}

				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_video
	} // class_exists	

	// add_filter('wp_video_shortcode', 'gdlr_core_fix_html5_autoplay', 10, 5);
	// if( !function_exists('gdlr_core_fix_html5_autoplay') ){
	// 	function gdlr_core_fix_html5_autoplay( $output, $atts, $video, $post_id, $library ){
	// 		if( !empty($atts['autoplay']) ){
	// 			$output = str_replace('autoplay', 'muted="1" autoplay', $output);
	// 		}
	// 		return $output;
	// 	}
	// }
