<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('social-network', 'gdlr_core_pb_element_social_network'); 
	
	if( !class_exists('gdlr_core_pb_element_social_network') ){
		class gdlr_core_pb_element_social_network{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'social_facebook',
					'title' => esc_html__('Social Network', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'icon-type' => array(
								'title' => esc_html__('Icon Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'font-awesome' => esc_html__('Font Awesome', 'goodlayers-core'),
									'font-awesome5' => esc_html__('Font Awesome 5', 'goodlayers-core'),
									'font-awesome6' => esc_html__('Font Awesome 6', 'goodlayers-core'),
								)
							),
							'tiktok' => array(
								'title' => esc_html__('Tiktok', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'icon-type' => 'font-awesome5' )
							),
							'discord' => array(
								'title' => esc_html__('Social Discord Link', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'icon-type' => 'font-awesome5' )
							),
							'twitch' => array(
								'title' => esc_html__('Social Twitch Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'delicious' => array(
								'title' => esc_html__('Social Delicious Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'email' => array(
								'title' => esc_html__('Social Email Link', 'goodlayers-core'),
								'type' => 'text',
								'default' => '#'
							),
							'deviantart' => array(
								'title' => esc_html__('Social Deviantart Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'digg' => array(
								'title' => esc_html__('Social Digg Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'facebook' => array(
								'title' => esc_html__('Social Facebook Link', 'goodlayers-core'),
								'type' => 'text',
								'default' => '#'
							),
							'flickr' => array(
								'title' => esc_html__('Social Flickr Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'lastfm' => array(
								'title' => esc_html__('Social Lastfm Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'linkedin' => array(
								'title' => esc_html__('Social Linkedin Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'pinterest' => array(
								'title' => esc_html__('Social Pinterest Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'rss' => array(
								'title' => esc_html__('Social RSS Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'skype' => array(
								'title' => esc_html__('Social Skype Link', 'goodlayers-core'),
								'type' => 'text',
								'default' => '#'
							),
							'stumbleupon' => array(
								'title' => esc_html__('Social Stumbleupon Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'tumblr' => array(
								'title' => esc_html__('Social Tumblr Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'twitter' => array(
								'title' => esc_html__('Social Twitter Link', 'goodlayers-core'),
								'type' => 'text',
								'default' => '#'
							),
							'vimeo' => array(
								'title' => esc_html__('Social Vimeo Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'youtube' => array(
								'title' => esc_html__('Social Youtube Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'dribbble' => array(
								'title' => esc_html__('Social Dribbble Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'behance' => array(
								'title' => esc_html__('Social Behance Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'instagram' => array(
								'title' => esc_html__('Social Instagram Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'snapchat' => array(
								'title' => esc_html__('Social Snapchat Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'whatsapp' => array(
								'title' => esc_html__('Social Whatsapp Link', 'goodlayers-core'),
								'type' => 'text'
							),

						)
					),
					'style' => array(
						'title' => esc_html__('Style & Size', 'goodlayers-core'),
						'options' => array(

							'direction' => array(
								'title' => esc_html__('Direction', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'horizontal' => esc_html__('Horizontal', 'goodlayers-core'),
									'vertical' => esc_html__('Vertical', 'goodlayers-core'),
								)
							),
							'text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'left'
							),	
							'icon-size' => array(
								'title' => esc_html__('Icon Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),	
							'with-text' => array(
								'title' => esc_html__('With Text', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),		
							'text-size' => array(
								'title' => esc_html__('Text Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '15px',
								'condition' => array( 'with-text' => 'enable' )
							),		
							'icon-border-size' => array(
								'title' => esc_html__('Icon Border Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							)

						)
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'background-color' => array(
								'title' => esc_html__('Icon Background', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'icon-color' => array(
								'title' => esc_html__('Icon Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'icon-border-color' => array(
								'title' => esc_html__('Icon Border Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'hover-background-color' => array(
								'title' => esc_html__('Icon Hover Background', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'hover-icon-color' => array(
								'title' => esc_html__('Icon Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'hover-icon-border-color' => array(
								'title' => esc_html__('Icon Hover Border Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							)
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'icon-width' => array(
								'title' => esc_html__('Icon Width & Line Height', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),	
							'icon-padding' => array(
								'title' => esc_html__('Icon Padding', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),	
							'icon-border-radius' => array(
								'title' => esc_html__('Icon Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),	
							'icon-space' => array(
								'title' => esc_html__('Space Between Icon', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '20px'
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
						'text-align' => 'left', 'social-head' => 'counter',
						'email' => '#', 'facebook' => '#', 'skype' => '#', 'twitter' => '#', 
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				// extra css
				$custom_style = '';
				if( !empty($settings['hover-background-color']) ){
					$custom_style .= '#custom_style_id.gdlr-core-social-network-item .gdlr-core-social-network-icon:hover{ background-color: ' . $settings['hover-background-color'] . ' !important; }';
				}
				if( !empty($settings['hover-icon-color']) ){
					$custom_style .= '#custom_style_id.gdlr-core-social-network-item .gdlr-core-social-network-icon:hover{ color: ' . $settings['hover-icon-color'] . ' !important; }';
				}
				if( !empty($settings['hover-icon-border-color']) ){
					$custom_style .= '#custom_style_id.gdlr-core-social-network-item .gdlr-core-social-network-icon:hover{ border-color: ' . $settings['hover-icon-border-color'] . ' !important; }';
				}
				if( !empty($custom_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_social_network_id;

						if( $preview ){
							$gdlr_core_social_network_id = empty($gdlr_core_social_network_id)? array(): $gdlr_core_social_network_id;

							// generate unique id so it does not get overwritten in admin area
							$rnd_social_network_id = mt_rand(0, 99999);
							while( in_array($rnd_social_network_id, $gdlr_core_social_network_id) ){
								$rnd_social_network_id = mt_rand(0, 99999);
							}
							$gdlr_core_social_network_id[] = $rnd_social_network_id;
							$settings['id'] = 'gdlr-core-social-network-' . $rnd_social_network_id;
						}else{
							$gdlr_core_social_network_id = empty($gdlr_core_social_network_id)? 1: $gdlr_core_social_network_id + 1;
							$settings['id'] = 'gdlr-core-social-network-' . $gdlr_core_social_network_id;
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
				$extra_class  = ' gdlr-core-' . (empty($settings['text-align'])? 'left': $settings['text-align']) . '-align';
				$extra_class .= empty($settings['no-pdlr'])? ' gdlr-core-item-pdlr': '';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$extra_class .= ' gdlr-direction-' . (empty($settings['direction'])? 'horizontal': $settings['direction']);
				
				$is_shortcode = empty($settings['is_shortcode'])? false: true; 

				$ret  = $is_shortcode? '<span ': '<div ';
				$ret .= 'class="gdlr-core-social-network-item gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				$ret .= gdlr_core_esc_style(array(
					'padding-bottom' => (empty($settings['padding-bottom']) || $settings['padding-bottom'] == $gdlr_core_item_pdb)? '': $settings['padding-bottom'],
					'display' => $is_shortcode? 'block': ''
				));
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( empty($settings['icon-type']) || $settings['icon-type'] == 'font-awesome' ){
					$social_list = array(
						'delicious' => array('title'=> 'Delicious', 'icon'=>'fa fa-delicious'), 
						'email' => array('title'=> 'Email', 'icon'=>'fa fa-envelope'),
						'deviantart' => array('title'=> 'Deviantart', 'icon'=>'fa fa-deviantart'),
						'digg' => array('title'=> 'Digg', 'icon'=>'fa fa-digg'),
						'facebook' => array('title'=> 'Facebook', 'icon'=>'fa fa-facebook'),
						'flickr' => array('title'=> 'Flickr', 'icon'=>'fa fa-flickr'),
						'lastfm' => array('title'=> 'Lastfm', 'icon'=>'fa fa-lastfm'),
						'linkedin' => array('title'=> 'Linkedin', 'icon'=>'fa fa-linkedin'),
						'pinterest' => array('title'=> 'Pinterest', 'icon'=>'fa fa-pinterest-p'),
						'rss' => array('title'=> 'Rss', 'icon'=>'fa fa-rss'), 
						'skype' => array('title'=> 'Skype', 'icon'=>'fa fa-skype'),
						'stumbleupon' => array('title'=> 'Stumbleupon', 'icon'=>'fa fa-stumbleupon'),
						'tumblr' => array('title'=> 'Tumblr', 'icon'=>'fa fa-tumblr'),
						'twitter' => array('title'=> 'Twitter', 'icon'=>'fa fa-twitter'),
						'vimeo' => array('title'=> 'Vimeo', 'icon'=>'fa fa-vimeo'),
						'youtube' => array('title'=> 'Youtube', 'icon'=>'fa fa-youtube'),
						'dribbble' => array('title'=> 'Dribbble', 'icon'=>'fa fa-dribbble'),
						'behance' => array('title'=> 'Behance', 'icon'=>'fa fa-behance'),
						'instagram' => array('title'=> 'Instagram', 'icon'=>'fa fa-instagram'),
						'snapchat' => array('title'=> 'Snapchat', 'icon'=>'fa fa-snapchat-ghost'),
						'twitch' => array('title'=> 'Twitch', 'icon'=>'fa fa-twitch'),
						'whatsapp' => array('title'=> 'Whatsapp', 'icon'=>'fa fa-whatsapp'),
					);
				}else if( $settings['icon-type'] == 'font-awesome5' ){
					$social_list = array(
						'tiktok' => array('title'=> 'Tiktok', 'icon'=>'fa5b fa5-tiktok'), 
						'delicious' => array('title'=> 'Delicious', 'icon'=>'fa5b fa5-delicious'), 
						'email' => array('title'=> 'Email', 'icon'=>'fa5s fa5-envelope'),
						'deviantart' => array('title'=> 'Deviantart', 'icon'=>'fa5b fa5-deviantart'),
						'digg' => array('title'=> 'Digg', 'icon'=>'fa5b fa5-digg'),
						'facebook' => array('title'=> 'Facebook', 'icon'=>'fa5b fa5-facebook'),
						'flickr' => array('title'=> 'Flickr', 'icon'=>'fa5b fa5-flickr'),
						'lastfm' => array('title'=> 'Lastfm', 'icon'=>'fa5b fa5-lastfm'),
						'linkedin' => array('title'=> 'Linkedin', 'icon'=>'fa5b fa5-linkedin'),
						'pinterest' => array('title'=> 'Pinterest', 'icon'=>'fa5b fa5-pinterest-p'),
						'rss' => array('title'=> 'Rss', 'icon'=>'fa5s fa5-rss'), 
						'skype' => array('title'=> 'Skype', 'icon'=>'fa5b fa5-skype'),
						'stumbleupon' => array('title'=> 'Stumbleupon', 'icon'=>'fa5b fa5-stumbleupon'),
						'tumblr' => array('title'=> 'Tumblr', 'icon'=>'fa5b fa5-tumblr'),
						'twitter' => array('title'=> 'Twitter', 'icon'=>'fa5b fa5-twitter'),
						'vimeo' => array('title'=> 'Vimeo', 'icon'=>'fa5b fa5-vimeo'),
						'youtube' => array('title'=> 'Youtube', 'icon'=>'fa5b fa5-youtube'),
						'dribbble' => array('title'=> 'Dribbble', 'icon'=>'fa5b fa5-dribbble'),
						'behance' => array('title'=> 'Behance', 'icon'=>'fa5b fa5-behance'),
						'instagram' => array('title'=> 'Instagram', 'icon'=>'fa5b fa5-instagram'),
						'snapchat' => array('title'=> 'Snapchat', 'icon'=>'fa5b fa5-snapchat-ghost'),
						'discord' => array('title'=> 'Discord', 'icon'=>'fa5b fa5-discord'),
						'twitch' => array('title'=> 'Twitch', 'icon'=>'fa5b fa5-twitch'),
						'whatsapp' => array('title'=> 'Whatsapp', 'icon'=>'fa5b fa5-whatsapp'),
					);
				}else if( $settings['icon-type'] == 'font-awesome6' ){
					$social_list = array(
						'tiktok' => array('title'=> 'Tiktok', 'icon'=>'fa6b fa6-tiktok'), 
						'delicious' => array('title'=> 'Delicious', 'icon'=>'fa6b fa6-delicious'), 
						'email' => array('title'=> 'Email', 'icon'=>'fa6s fa6-envelope'),
						'deviantart' => array('title'=> 'Deviantart', 'icon'=>'fa6b fa6-deviantart'),
						'digg' => array('title'=> 'Digg', 'icon'=>'fa6b fa6-digg'),
						'facebook' => array('title'=> 'Facebook', 'icon'=>'fa6b fa6-facebook'),
						'flickr' => array('title'=> 'Flickr', 'icon'=>'fa6b fa6-flickr'),
						'lastfm' => array('title'=> 'Lastfm', 'icon'=>'fa6b fa6-lastfm'),
						'linkedin' => array('title'=> 'Linkedin', 'icon'=>'fa6b fa6-linkedin'),
						'pinterest' => array('title'=> 'Pinterest', 'icon'=>'fa6b fa6-pinterest-p'),
						'rss' => array('title'=> 'Rss', 'icon'=>'fa6s fa6-rss'), 
						'skype' => array('title'=> 'Skype', 'icon'=>'fa6b fa6-skype'),
						'stumbleupon' => array('title'=> 'Stumbleupon', 'icon'=>'fa6b fa6-stumbleupon'),
						'tumblr' => array('title'=> 'Tumblr', 'icon'=>'fa6b fa6-tumblr'),
						'twitter' => array('title'=> 'Twitter', 'icon'=>'fa6b fa6-x-twitter'),
						'vimeo' => array('title'=> 'Vimeo', 'icon'=>'fa6b fa6-vimeo'),
						'youtube' => array('title'=> 'Youtube', 'icon'=>'fa6b fa6-youtube'),
						'dribbble' => array('title'=> 'Dribbble', 'icon'=>'fa6b fa6-dribbble'),
						'behance' => array('title'=> 'Behance', 'icon'=>'fa6b fa6-behance'),
						'instagram' => array('title'=> 'Instagram', 'icon'=>'fa6b fa6-instagram'),
						'snapchat' => array('title'=> 'Snapchat', 'icon'=>'fa6b fa6-snapchat-ghost'),
						'discord' => array('title'=> 'Discord', 'icon'=>'fa6b fa6-discord'),
						'twitch' => array('title'=> 'Twitch', 'icon'=>'fa6b fa6-twitch'),
						'whatsapp' => array('title'=> 'Whatsapp', 'icon'=>'fa6b fa6-whatsapp'),
					);
				}
				

				$settings['icon-space'] = (empty($settings['icon-space']) || $settings['icon-space'] == '20px')? '': $settings['icon-space'];
				$margin_direction = (empty($settings['direction']) || $settings['direction'] == 'horizontal' )? 'margin-right': 'margin-bottom';
				foreach( $social_list as $social_key => $social_option ){
					$social_title = $social_option['title'];
					$social_icon = $social_option['icon'];

					if( !empty($settings[$social_key]) ){
						$social_link = $settings[$social_key];

						if( $social_key == 'email' && !empty($social_link) ){
							$social_link = 'mailto:' . $social_link;
						}

						$ret .= '<a href="' . esc_url($social_link) . '" ';
						$ret .= 'target="' . (empty($settings['link-target'])? '_blank': $settings['link-target']) . '" ';
						$ret .= 'class="gdlr-core-social-network-icon" title="' . esc_attr($social_key) . '" ' . gdlr_core_esc_style(array(
							'font-size' => empty($settings['icon-size'])? '': $settings['icon-size'],
							'color' => empty($settings['icon-color'])? '': $settings['icon-color'],
							'border-color' => empty($settings['icon-border-color'])? '': $settings['icon-border-color'],
							'border-width' => empty($settings['icon-border-size'])? '': $settings['icon-border-size'],
							$margin_direction => $settings['icon-space'],
							'width' => empty($settings['icon-width'])? '': $settings['icon-width'],
							'height' => empty($settings['icon-width'])? '': $settings['icon-width'],
							'line-height' => empty($settings['icon-width'])? '': $settings['icon-width'],
							'background-color' => empty($settings['background-color'])? '': $settings['background-color'],
							'border-radius' => empty($settings['icon-border-radius'])? '': $settings['icon-border-radius'],
						)) . ' >';
						$ret .= '<i class="' . esc_attr($social_icon) . '" ></i>';
						if( !empty($settings['with-text']) && $settings['with-text'] == 'enable' ){
							$ret .= '<span class="gdlr-core-social-network-item-text" ' . gdlr_core_esc_style(array(
								'font-size' => (empty($settings['text-size']) || $settings['text-size'] == '15px')? '': $settings['text-size']
							)) . ' >' . $social_title . '</span>';
						}
						$ret .= '</a>';
					}
				}

				$ret .= $is_shortcode? '</span>': '</div>';
				$ret .= $custom_style;
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_social_share
	} // class_exists	

	add_shortcode('gdlr_core_social_network', 'gdlr_core_social_network_shortcode');
	if( !function_exists('gdlr_core_social_network_shortcode') ){
		function gdlr_core_social_network_shortcode($atts){
			$atts = wp_parse_args($atts, array(
				'no-pdlr' => true,
				'padding-bottom' => '0px',
				'text-align' => 'none',
				'icon-size' => '',
				'is_shortcode' => true
			));

			return gdlr_core_pb_element_social_network::get_content($atts);
		}
	}
