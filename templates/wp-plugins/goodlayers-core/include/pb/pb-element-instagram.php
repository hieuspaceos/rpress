<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('instagram', 'gdlr_core_pb_element_instagram');

	if( !class_exists('gdlr_core_pb_element_instagram') ){
		class gdlr_core_pb_element_instagram{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-instagram',
					'title' => esc_html__('Instagram', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'title' => array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'type' => 'text',
							),
							'username' => array(
								'title' => esc_html__('Instagram Username', 'goodlayers-core'),
								'type' => 'text',
							),
							'client-id' => array(
								'title' => esc_html__('Access Token', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('You can see how to obtain the client-id here.') . ' ' .
									'<a href="https://developers.facebook.com/docs/instagram-basic-display-api/getting-started/" target="_blank" >https://developers.facebook.com/docs/instagram-basic-display-api/getting-started/</a>'
							),
							'num-fetch' => array(
								'title' => esc_html__('Display Number', 'goodlayers-core'), 
								'type' => 'text',
								'default' => 9,
								'description' => esc_html__('Maximum number is 20', 'goodlayers-core')
							),
							'cache-time' => array(
								'title' => esc_html__('Cache Time (Hours)', 'goodlayers-core'),
								'type' => 'text',
								'default' => 1
							),
						)
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'layout' => array(
								'title' => esc_html__('Layout', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'list' => esc_html__('List', 'goodlayers-core'),
									'list-no-space' => esc_html__('List No Space', 'goodlayers-core'),
									'carousel' => esc_html__('Carousel', 'goodlayers-core'),
									'carousel-no-space' => esc_html__('Carousel No Space', 'goodlayers-core'),
								)
							),
							'carousel-autoslide' => array(
								'title' => esc_html__('Autoslide Carousel', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array( 'layout' => array('carousel', 'carousel-no-space') )
							),
							'column' => array(
								'title' => esc_html__('Column Number', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 1=>1, 2=>2, 3=>3, 4=>4, 5=>5, 6=>6),
								'default' => 3
							),
							'centering-image' => array(
								'title' => esc_html__('Centering Image (If images is not in the same proportion)', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
						)
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'title-size' => array(
								'title' => esc_html__('Title Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '24px'
							),
						)
					),

					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'title-wrap-bottom-margin' => array(
								'title' => esc_html__('Title Wrap Bottom Margin', 'goodlayers-core'),
								'type' => 'text',
								'default' => '30px',
								'data-input-type' => 'pixel',
							),
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom ( Item )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							)
						)
					),
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-instagram-<?php echo esc_attr($id); ?>" >
if( document.readyState == 'complete' ){
	jQuery(document).ready(function(){
		jQuery('#gdlr-core-preview-instagram-<?php echo esc_attr($id); ?>').parent().gdlr_core_flexslider();
	});
}else{
	jQuery(window).on('load', function(){
		jQuery('#gdlr-core-preview-instagram-<?php echo esc_attr($id); ?>').parent().gdlr_core_flexslider();
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
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				$settings['layout'] = empty($settings['layout'])? 'list': $settings['layout'];
				$no_space = (strpos($settings['layout'], '-no-space') !== false)? 'yes': 'no';
				$settings['layout'] = str_replace('-no-space', '', $settings['layout']);
				
				// start printing item
				$extra_class  = empty($settings['class'])? '': ' ' . $settings['class'];
				$extra_class .= ($no_space == 'yes' || $settings['layout'] == 'carousel')? ' gdlr-core-item-pdlr': '';

				$ret  = '<div class="gdlr-core-instagram-item gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( !empty($settings['username']) && !empty($settings['client-id']) ){

					$settings['cache-time'] = empty($settings['cache-time'])? 1: $settings['cache-time'];
					$settings['num-fetch'] = empty($settings['num-fetch'])? 9: $settings['num-fetch'];

					$instagram_images = self::get_instagram($settings['username'], $settings['client-id'], $settings['num-fetch'], $settings['cache-time']);

					if( is_wp_error($instagram_images) ){
						$ret .= '<div class="gdlr-core-external-plugin-message">' . $instagram_images->get_error_message() . '</div>';
					}else{

						// display title
						if( !empty($settings['title']) ){
							$title_class = ($no_space == 'yes' || $settings['layout'] == 'carousel')? '': ' gdlr-core-item-pdlr'; 
							$ret .= '<div class="gdlr-core-instagram-item-title-wrap ' . esc_attr($title_class) . '" ' . gdlr_core_esc_style(array(
								'margin-bottom' => empty($settings['title-wrap-bottom-margin'])? '': $settings['title-wrap-bottom-margin'] 
							)) . ' >';
							$ret .= '<h3 class="gdlr-core-instagram-item-title" ' . gdlr_core_esc_style(array(
								'font-size' => empty($settings['title-size'])? '': $settings['title-size'] 
							)) . ' >' . $settings['title'] . '</h3>';

							if( $settings['layout'] == 'carousel' ){
								$ret .= '<div class="gdlr-core-instagram-item-title-nav" >';
								$ret .= '<i class="flex-prev fa fa-angle-left" ></i>';
								$ret .= '<a href="https://www.instagram.com/' . esc_attr($settings['username']) . '" target="_blank" >';
								$ret .= '<i class="fa fa-instagram" ></i>';
								$ret .= '</a>';
								$ret .= '<i class="flex-next fa fa-angle-right" ></i>';
								$ret .= '</div>';
							}
							$ret .= '</div>';
						}

						// start displaying the data
						$column = empty($settings['column'])? 3: $settings['column'];
						// $thumbnail_size = empty($settings['thumbnail-size'])? 'thumbnail': $settings['thumbnail-size'];
						$lightbox_group = gdlr_core_image_group_id();
						$centering_image = (!empty($settings['centering-image']) && $settings['centering-image'] == 'enable')? true: false;

						if( $settings['layout'] == 'list' ){

							$column_count = 0;
							$column_size = 60 / intval($column);

							$ret .= '<div class="gdlr-core-instagram-item-content clearfix" >';
							foreach( $instagram_images as $image ){

								if( empty($image['media_url']) && !empty($image['large']['url']) ){
									$image['media_url'] = $image['large']['url'];
								}

								if( !empty($image['media_url']) ){
									$column_class  = ' gdlr-core-column-' . $column_size;
									$column_class .= ($column_count % 60 == 0)? ' gdlr-core-column-first': '';
									$column_class .= ($no_space == 'yes')? '': ' gdlr-core-item-pdlr gdlr-core-item-mgb';
									$column_class .= ($centering_image)? ' gdlr-core-js': '';

									$ret .= '<div class="' . esc_attr($column_class) . ' gdlr-core-media-image" ';
									$ret .= ($centering_image)? ' data-sync-height="gdlr-core-instagram" data-sync-height-center ': '';
									$ret .= ' >';
									$ret .= '<a ' . gdlr_core_get_lightbox_atts(array(
										'url'=>$image['media_url'], 
										'group' => $lightbox_group,
										'caption' => (empty($image['caption'])? '': esc_attr($image['caption']))
									)) . ' >';
									$ret .= '<img src="' . esc_url($image['media_url']) . '" ';
									// $ret .= empty($image[$thumbnail_size]['width'])? '': ' width="' . esc_attr($image[$thumbnail_size]['width']) . '" ';
									// $ret .= empty($image[$thumbnail_size]['height'])? '': ' height="' . esc_attr($image[$thumbnail_size]['height']) . '" ';
									$ret .= ' alt="' . (empty($image['caption'])? '': esc_attr($image['caption'])) . '" ';
									$ret .= ' />';
									$ret .= '</a>';
									$ret .= '</div>';

									$column_count += $column_size;
								}
							}
							$ret .= '</div>';

						}else if( $settings['layout'] == 'carousel' ){
							
							$slides = array();
							$flex_atts = array(
								'carousel' => true,
								'column' => $column,
								'navigation' => 'navigation',
								'nav-parent' => 'gdlr-core-instagram-item',
								'nav-type' => 'custom',
								'mglr' => ($no_space == 'no'),
								'disable-autoslide' => (empty($settings['carousel-autoslide']) || $settings['carousel-autoslide'] == 'enable')? '': true,
							);

							foreach( $instagram_images as $image ){

								if( empty($image['media_url']) && !empty($image['large']['url']) ){
									$image['media_url'] = $image['large']['url'];
								}

								if( !empty($image['media_url']) ){

									$slide  = '<div class="gdlr-core-media-image" >';
									$slide .= '<a ' . gdlr_core_get_lightbox_atts(array(
										'url'=>$image['media_url'], 
										'group' => $lightbox_group,
										'caption' => (empty($image['caption'])? '': esc_attr($image['caption']))
									)) . ' >';
									$slide .= '<img src="' . esc_url($image['media_url']) . '" ';
									// $slide .= empty($image[$thumbnail_size]['width'])? '': ' width="' . esc_attr($image[$thumbnail_size]['width']) . '" ';
									// $slide .= empty($image[$thumbnail_size]['height'])? '': ' height="' . esc_attr($image[$thumbnail_size]['height']) . '" ';
									$slide .= ' alt="' . (empty($image['caption'])? '': esc_attr($image['caption'])) . '" ';
									$slide .= ' />';
									$slide .= '</a>';
									$slide .= '</div>';

									$slides[] = $slide;
								}
							}

							$ret .= gdlr_core_get_flexslider($slides, $flex_atts);
						}
					}
					
				}else{
					$ret .= '<div class="gdlr-core-external-plugin-message">' . esc_html__('Cannot retrieve instagram data, please fill in instagram Username and Access Token.', 'goodlayers-core-twitter') . '</div>';
				}

				$ret .= '</div>'; // gdlr-core-instagram-item
				
				return $ret;
			}

			static function get_instagram( $username, $access_token, $num_fetch = 9, $cache_time = 1 ){

				$transient_slug = 'gdlr-core-instagram-' . $username . '-' . $num_fetch;
				$instagram_images = get_option($transient_slug, array());
				if( empty($instagram_images) ){
					$instagram_images = get_transient($transient_slug);
					if( !empty($instagram_images) ){
						update_option($transient_slug, $instagram_images);
					}
				}

				$current_time = strtotime("now");
				$old_cache_time = get_option($transient_slug . '-time', $current_time);
				
				if( empty($instagram_images) || $old_cache_time + (3600 * $cache_time) < $current_time ){ 

					// obtain an images
					$remote = wp_remote_get('https://graph.instagram.com/me/media?fields=media_url,media_type,caption,thumbnail_url&limit=' . trim($num_fetch) . '&access_token=' . trim($access_token));
						
					if( is_wp_error($remote) ){
						if( !empty($instagram_images) ){
							return $instagram_images;
						}else{
							return new WP_Error('site_down', __('Unable to communicate with Instagram.', 'gdlr_translate'));
						}
					}

					if( 200 != wp_remote_retrieve_response_code($remote) ){
						if( !empty($instagram_images) ){
							return $instagram_images;
						}else{
							$message = '';
							$user_content = json_decode($remote['body'], true);
							if( !empty($user_content['error']['message']) ){
								$message = $user_content['error']['message'];
							}
							return new WP_Error('invalid_response', __('Instagram did not return a 200.', 'gdlr_translate') . ' ' . $message);
						}
					}

					$user_content = json_decode($remote['body'], true);
					$images = $user_content['data'];

					$instagram_images = array();
					foreach( $images as $image ) {

						if( in_array($image['media_type'], array('IMAGE','CAROUSEL_ALBUM')) ){
							$instagram_images[] = array(
								'id' => empty($image['id'])? '':$image['id'],
								'caption' => empty($image['caption'])? '':$image['caption'],
								'media_url' => empty($image['media_url'])? '':$image['media_url']
							);
						}
					}

					update_option($transient_slug, $instagram_images);
					update_option($transient_slug . '-time', $current_time);
				}

				return $instagram_images;
			}
			
		} // gdlr_core_pb_element_instagram
	} // class_exists	