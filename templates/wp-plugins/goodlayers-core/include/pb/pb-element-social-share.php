<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('social-share', 'gdlr_core_pb_element_social_share'); 
	
	if( !class_exists('gdlr_core_pb_element_social_share') ){
		class gdlr_core_pb_element_social_share{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'social_twitter',
					'title' => esc_html__('Social Share', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'facebook' => array(
								'title' => esc_html__('Facebook', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable'
							),
							'facebook-access-token' => array(
								'title' => esc_html__('Facebook Access Token', 'goodlayers-core'),
								'type' => 'text',
								'description' => wp_kses(__('You can fill the access token as "app_id|secret_key".', 'goodlayers-core'), array('a' => array( 'href' => array(), 'target' => array() ) ))
							),
							'linkedin' => array(
								'title' => esc_html__('Linkedin', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),			
							'pinterest' => array(
								'title' => esc_html__('Pinterest', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable'
							),			
							'stumbleupon' => array(
								'title' => esc_html__('Stumbleupon', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),			
							'twitter' => array(
								'title' => esc_html__('Twitter', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable'
							),			
							'email' => array(
								'title' => esc_html__('Email', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'social-head' => array(
								'title' => esc_html__('Social Head Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'counter' => esc_html__('Counter', 'goodlayers-core'),
									'custom-text' => esc_html__('Custom Text', 'goodlayers-core'),
								),
								'default' => 'counter',
								'description' => esc_html__('* Counter option will cache the value for about 1 hour, so the share count will not be showing right away.', 'goodlayers-core'),
							),
							'social-head-text' => array(
								'title' => esc_html__('Social Head Text', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'social-head' => 'custom-text' ),
								'default' => esc_html__('Social Shares', 'goodlayers-core'),
							)
						)
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'style' => array(
								'title' => esc_html__('Social Share Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'plain' => esc_html__('Plain', 'goodlayers-core'),
									'round' => esc_html__('Round', 'goodlayers-core'),
									'color' => esc_html__('Color', 'goodlayers-core')
								),
								'default' => 'plain',
							),
							'layout' => array(
								'title' => esc_html__('Social Share Layout', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'left-text' => GDLR_CORE_URL . '/include/images/social-share/left-text.png',
									'right-text' => GDLR_CORE_URL . '/include/images/social-share/right-text.png',
									'top-text' => GDLR_CORE_URL . '/include/images/social-share/top-text.png',
									'vertical' => GDLR_CORE_URL . '/include/images/social-share/vertical.png',
								),
								'default' => 'left-text',
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'text-align' => array(
								'title' => esc_html__('Alignment', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'left'
							)
						)
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
								'icon-size' => array(
								'title' => esc_html__('Icon Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '16px'
							)
						)
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'text-color' => array(
								'title' => esc_html__('Text Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),			
							'icon-color' => array(
								'title' => esc_html__('Icon Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),		
							'icon-background' => array(
								'title' => esc_html__('Icon Background', 'goodlayers-core'),
								'type' => 'colorpicker',
								'description' => esc_html__('For round style', 'goodlayers-core')
							),			
							'divider-color' => array(
								'title' => esc_html__('Divider Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							)
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'icon-space' => array(
								'title' => esc_html__('Space Between Icon', 'goodlayers-core'),
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
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'layout' => 'left-text', 'text-align' => 'left', 'social-head' => 'counter',
						'facebook' => 'enable', 'pinterest' => 'enable', 'twitter' => 'enable', 
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				// default value
				$settings['icon-size'] = (empty($settings['icon-size']) || $settings['icon-size'] == '16px')? '': $settings['icon-size'];
				$settings['layout'] = empty($settings['layout'])? 'left-text': $settings['layout'];
				$settings['social-head'] = empty($settings['social-head'])? 'counter': $settings['social-head'];
				
				$url = get_permalink();
				$page_id = get_the_ID();
				$title = urlencode(html_entity_decode(get_the_title(), ENT_COMPAT, 'UTF-8'));
				
				// get the count number
				if( $settings['social-head'] == 'counter'){
					if( $preview ){
						$social_count = 0;
					}else{
						$social_count = get_transient('gdlr-core-social-share-' . $page_id);
						if( $social_count === false || isset($_GET['refresh-social-count']) ){
							$social_list = array();
							if( !empty($settings['facebook']) && $settings['facebook'] == 'enable' ){ $social_list[] = 'facebook'; }
							if( !empty($settings['linkedin']) && $settings['linkedin'] == 'enable' ){ $social_list[] = 'linkedin'; }
							if( !empty($settings['pinterest']) && $settings['pinterest'] == 'enable' ){ $social_list[] = 'pinterest'; }
							if( !empty($settings['stumbleupon']) && $settings['stumbleupon'] == 'enable' ){ $social_list[] = 'stumbleupon'; }
							if( !empty($settings['twitter']) && $settings['twitter'] == 'enable' ){ $social_list[] = 'twitter'; }
							$social_count = gdlr_core_get_social_count($social_list, $url, $settings);

							$expiration = 86400;
							set_transient('gdlr-core-social-share-' . $page_id, $social_count, $expiration);
						}
					}
				}

				// start printing item
				$extra_class  = ' gdlr-core-' . (empty($settings['text-align'])? 'left': $settings['text-align']) . '-align';
				$extra_class .= ' gdlr-core-social-share-' . $settings['layout'];
				$extra_class .= (empty($settings['no-pdlr']))? ' gdlr-core-item-mglr': '';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$extra_class .= ' gdlr-core-style-' . (empty($settings['style'])? 'plain': $settings['style']);
				$extra_class .= ($settings['social-head'] == 'none')? ' gdlr-core-no-counter ': ''; 
				$ret  = '<div class="gdlr-core-social-share-item gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				// social count area
				if( $settings['social-head'] != 'none' && $settings['layout'] != 'right-text' ){
					$ret .= '<span class="gdlr-core-social-share-count gdlr-core-skin-title" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['text-color'])? '': $settings['text-color']
					)) . ' >';
					if( $settings['social-head'] == 'counter' ){
						$ret .= '<span class="gdlr-core-count" >' . gdlr_core_escape_content($social_count) . '</span>';
						$ret .= '<span class="gdlr-core-suffix" >' . esc_html__('Shares', 'goodlayers-core') . '</span>';
					}else if( $settings['social-head'] == 'custom-text' && !empty($settings['social-head-text']) ){
						$ret .= '<span class="gdlr-core-suffix" >' . gdlr_core_text_filter($settings['social-head-text']) . '</span>';
					}
					$ret .= '<span class="gdlr-core-divider gdlr-core-skin-divider" ' . gdlr_core_esc_style(array(
						'border-color' => empty($settings['divider-color'])? '': $settings['divider-color']
					)) . ' ></span>';
					$ret .= '</span>';
				}

				// social icon area
				$icon_style = array(
					'font-size' => $settings['icon-size'],
					'color' => empty($settings['icon-color'])? '': $settings['icon-color'],
					
				);
				if( $settings['layout'] == 'vertical' ){
					$icon_style['margin-bottom'] = empty($settings['icon-space'])? '': $settings['icon-space'];
				}else{
					$icon_style['margin-left'] = empty($settings['icon-space'])? '': $settings['icon-space'];
					$icon_style['margin-right'] = empty($settings['icon-space'])? '': $settings['icon-space'];
				}
				if( !empty($settings['style']) && $settings['style'] == 'round' ){
					$icon_style['background-color'] = empty($settings['icon-background'])? '': $settings['icon-background'];
				}
				$icon_style = gdlr_core_esc_style($icon_style);

				$ret .= '<span class="gdlr-core-social-share-wrap">';
				if( !empty($settings['facebook']) && $settings['facebook'] == 'enable' ){
					// $share_url = 'http://www.facebook.com/share.php?u=' . $url . '&title=' . $title;
					$share_url = 'https://www.facebook.com/sharer/sharer.php?caption=' . $title . '&u=' . $url;
					$onclick_event = 'javascript:window.open(this.href,\'\', \'menubar=no,toolbar=no,resizable=yes,scrollbars=yes,height=602,width=555\');return false;';

					$ret .= '<a class="gdlr-core-social-share-facebook" href="' . esc_url($share_url) . '" target="_blank" onclick="' . esc_attr($onclick_event) . '" ' . $icon_style . ' ><i class="fa fa-facebook" ></i></a>';
				}

				if( !empty($settings['linkedin']) && $settings['linkedin'] == 'enable' ){
					$share_url = 'http://www.linkedin.com/shareArticle?mini=true&url=' . $url . '&title=' . $title;
					$onclick_event = 'javascript:window.open(this.href,\'\', \'menubar=no,toolbar=no,resizable=yes,scrollbars=yes,height=452,width=550\');return false;';

					$ret .= '<a class="gdlr-core-social-share-linkedin" href="' . esc_url($share_url) . '" target="_blank" onclick="' . esc_attr($onclick_event) . '" ' . $icon_style . ' ><i class="fa fa-linkedin" ></i></a>';
				}

				if( !empty($settings['pinterest']) && $settings['pinterest'] == 'enable' ){
					$share_url = 'http://pinterest.com/pin/create/button/?url=' . $url;
					if( has_post_thumbnail() ){
						$share_url .= '&media=' . wp_get_attachment_url(get_post_thumbnail_id(), 'large');
					}
					$onclick_event = 'javascript:window.open(this.href,\'\', \'menubar=no,toolbar=no,resizable=yes,scrollbars=yes,height=553,width=750\');return false;';

					$ret .= '<a class="gdlr-core-social-share-pinterest" href="' . esc_url($share_url) . '" target="_blank" onclick="' . esc_attr($onclick_event) . '" ' . $icon_style . ' ><i class="fa fa-pinterest-p" ></i></a>';
				}

				if( !empty($settings['stumbleupon']) && $settings['stumbleupon'] == 'enable' ){
					$share_url = 'http://www.stumbleupon.com/submit?url=' . $url . '&title=' . $title;
					$onclick_event = 'javascript:window.open(this.href,\'\', \'menubar=no,toolbar=no,resizable=yes,scrollbars=yes,height=553,width=750\');return false;';

					$ret .= '<a class="gdlr-core-social-share-stumbleupon" href="' . esc_url($share_url) . '" target="_blank" onclick="' . esc_attr($onclick_event) . '" ' . $icon_style . ' ><i class="fa fa-stumbleupon" ></i></a>';
				}

				if( !empty($settings['twitter']) && $settings['twitter'] == 'enable' ){
					//$share_url = 'http://twitter.com/home?status=' . $title . '+' . $url;
					// $share_url = 'http://twitter.com/intent/tweet?text=' . $title . '+' . $url;
					$share_url = 'https://twitter.com/intent/tweet?text=' . $title . '&url=' . $url;
					$onclick_event = 'javascript:window.open(this.href,\'\', \'menubar=no,toolbar=no,resizable=yes,scrollbars=yes,height=255,width=555\');return false;';

					$ret .= '<a class="gdlr-core-social-share-twitter" href="' . esc_url($share_url) . '" target="_blank" onclick="' . esc_attr($onclick_event) . '" ' . $icon_style . ' ><i class="fa fa-twitter fa6b fa6-x-twitter" ></i></a>';
				}

				if( !empty($settings['email']) && $settings['email'] == 'enable' ){
					$mail_title = esc_html__('Site sharing', 'goodlayers-core');
					$mail_body = esc_html__('Please check this site out', 'goodlayers-core');
					$share_url = "mailto:?subject={$mail_title}&amp;body={$mail_body} {$url}";

					$ret .= '<a class="gdlr-core-social-share-email" href="' . esc_url($share_url) . '" ' . $icon_style . ' ><i class="fa fa-envelope" ></i></a>';
				}
				$ret .= '</span>'; // gdlr-core-social-share-wrap

				// social count (right) area
				if( $settings['social-head'] != 'none' && $settings['layout'] == 'right-text' ){
					$ret .= '<span class="gdlr-core-social-share-count" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['text-color'])? '': $settings['text-color']
					)) . '>';
					$ret .= '<span class="gdlr-core-divider gdlr-core-skin-divider" ' . gdlr_core_esc_style(array(
						'border-color' => empty($settings['divider-color'])? '': $settings['divider-color']
					)) . ' ></span>';
					if( $settings['social-head'] == 'counter' ){
						$ret .= '<span class="gdlr-core-count" >' . gdlr_core_escape_content($social_count) . '</span>';
						$ret .= '<span class="gdlr-core-suffix" >' . esc_html__('Shares', 'goodlayers-core') . '</span>';
					}else if( $settings['social-head'] == 'custom-text' && !empty($settings['social-head-text']) ){
						$ret .= '<span class="gdlr-core-suffix" >' . gdlr_core_text_filter($settings['social-head-text']) . '</span>';
					}
					$ret .= '</span>';
				}

				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_social_share
	} // class_exists	

	if( !function_exists('gdlr_core_get_social_count') ){
		function gdlr_core_get_social_count( $social_list = array(), $url = '', $settings = array() ){
			$count = 0;

			foreach( $social_list as $social ){
				switch( $social ){
					case 'facebook': 
						$settings['facebook-access-token'] = empty($settings['facebook-access-token'])? '': $settings['facebook-access-token'];

						$response = wp_remote_get('https://graph.facebook.com/?id=' . trim($url) . '&fields=engagement&access_token=' . trim($settings['facebook-access-token']));
						$response_body = wp_remote_retrieve_body($response);

						if( current_user_can('administrator') && isset($_GET['debug_social_count']) ){
							echo 'https://graph.facebook.com/?id=' . trim($url) . '&fields=engagement&access_token=' . trim($settings['facebook-access-token']);
						}

						$data = json_decode($response_body);
						if( !empty($data->engagement->share_count) ){
						 	$count += $data->engagement->share_count;
						}
						// if( !empty($data->share->share_count) ){
						// 	$count += $data->share->share_count;
						// }else if( !empty($data->shares) ){
						// 	$count += $data->shares;
						// }

						break;
					case 'linkedin': 
						$response = wp_remote_get('https://www.linkedin.com/countserv/count/share?url=' . $url . '&format=json');
						$response_body = wp_remote_retrieve_body($response);

						$data = json_decode($response_body);
						if( !empty($data->count) ){
							$count += $data->count;
						}

						break;
					case 'pinterest': 
						$response = wp_remote_get('https://api.pinterest.com/v1/urls/count.json?callback%20&url=' . $url);
						$response_body = wp_remote_retrieve_body($response);
						$response_body = preg_replace('/^receiveCount\((.*)\)$/', '\\1', $response_body);

						$data = json_decode($response_body);
						if( !empty($data->count) && is_numeric($data->count) ){
							$count += $data->count;
						}

						break;
					case 'stumbleupon': 
						$response = wp_remote_get('https://www.stumbleupon.com/services/1.01/badge.getinfo?url=' . $url);
						$response_body = wp_remote_retrieve_body($response);

						$data = json_decode($response_body);
						if( !empty($data->count) ){
							$count += $data->count;
						}

						break;
					case 'twitter': 
						// ref : http://opensharecount.com/
						$response = wp_remote_get('http://opensharecount.com/count.json?url=' . $url);
						$response_body = wp_remote_retrieve_body($response);

						$data = json_decode($response_body);
						if( !empty($data->count) ){
							$count += $data->count;
						}

						break;
					default: break;
				}
			}

			return $count;
		}
	}