<?php
	if( !function_exists('gdlr_core_get_privacy_options') ){
		function gdlr_core_get_privacy_options( $type = 1 ){

			if( $type == 1 ){
				return array(
					'consent-item' => array(
						'title' => esc_html__('Consent Item', 'goodlayers-core'),
						'type' => 'custom',
						'item-type' => 'tabs',
						'wrapper-class' => 'gdlr-core-fullsize',
						'options' => array(
							'sub-title' => array(
								'title' => esc_html__('Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'plain-text' => esc_html__('Plain Text', 'goodlayers-core'),
									'cookie-consent' => esc_html__('Cookie Consent', 'goodlayers-core')
								)
							),
							'title' => array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'type' => 'text'
							),
							'content' => array(
								'title' => esc_html__('Content', 'goodlayers-core'),
								'type' => 'tmce'
							),
							'default' => array(
								'title' => esc_html__('Default Value', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array('sub-title' => 'cookie-consent')
							),
							'required' => array(
								'title' => esc_html__('Required', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array('sub-title' => 'cookie-consent', 'default' => 'enable'),
							),
							'script-type' => array(
								'title' => esc_html__('Script Type', 'tourmaster'),
								'type' => 'combobox',
								'options' => array(
									'snippet' => esc_html__('Snippet', 'tourmaster'),
									'page-item-consent' => esc_html__('Page Item Consent', 'tourmaster'),
								),
								'condition' => array('sub-title' => 'cookie-consent')
							),
							'snippet' => array(
								'title' => esc_html__('Code Snippet (Analytics/Advertising Code)', 'tourmaster'),
								'type' => 'textarea',
								'condition' => array('sub-title' => 'cookie-consent', 'script-type' => 'snippet')
							),
							'placement' => array(
								'title' => esc_html__('Placement', 'tourmaster'),
								'type' => 'combobox',
								'options' => array(
									'head' => esc_html__('Within Head Tag', 'tourmaster'),
									'footer' => esc_html__('Within Footer Tag', 'tourmaster'),
								),
								'condition' => array('sub-title' => 'cookie-consent', 'script-type' => 'snippet')
							)
						),
						'description-position' => 'top',
						'description' => esc_html__('Note! this feature doesn\'t assure that your website will be 100% GDPR compliant. If you are not sure, you should contact law firm for more information.', 'goodlayers-core')
					)
				);
			}else{
				return array(
					'disable-cookie-youtube' => array(
						'title' => esc_html__('Disable Cookie On Youtube Videos', 'goodlayers-core'),
						'type' => 'checkbox',
						'default' => 'disable'
					),
					'privacy-cookie-time' => array(
						'title' => esc_html__('Cookie Time (# days)', 'goodlayers-core'),
						'type' => 'text',
						'default' => 30
					),
					'privacy-notification-position' => array(
						'title' => esc_html__('Privacy Box Position', 'goodlayers-core'),
						'type' => 'combobox',
						'options' => array(
							'top-bar' => esc_html__('Top Bar', 'goodlayers-core'),
							'bottom-bar' => esc_html__('Bottom Bar', 'goodlayers-core'),
							'bottom-left' => esc_html__('Bottom Left', 'goodlayers-core'),
							'bottom-right' => esc_html__('Bottom Right', 'goodlayers-core'),
							'none' => esc_html__('Do Not Show', 'goodlayers-core'),
						),
						'default' => 'bottom-bar'
					),
					'privacy-box-button-color' => array(
						'title' => esc_html__('Privacy Box Button Color', 'goodlayers-core'),
						'type' => 'colorpicker',
						'default' => '#1a49a8',
						'selector' => '.gdlr-core-body .gdlr-core-privacy-box-wrap .gdlr-core-privacy-box-button{ background: #gdlr#; }'
					),
					'privacy-form-logo' => array(
						'title' => esc_html__('Privacy Form Logo', 'goodlayers-core'),
						'type' => 'upload',
					),
					'privacy-form-input-button-color' => array(
						'title' => esc_html__('Privacy Form Input Button Color', 'goodlayers-core'),
						'type' => 'colorpicker',
						'default' => '#1a49a8',
						'selector' => '.gdlr-core-gdpr-form-checkbox:checked + .gdlr-core-gdpr-form-checkbox-appearance .gdlr-core-gdpr-form-checkbox-button{ background-color: #gdlr#; }' . 
							'.gdlr-core-gdpr-form-checkbox:checked + .gdlr-core-gdpr-form-checkbox-appearance + .gdlr-core-gdpr-form-checkbox-text,' . 
							'.gdlr-core-gdpr-form-checkbox-required + .gdlr-core-gdpr-form-checkbox-appearance + .gdlr-core-gdpr-form-checkbox-text{ color: #gdlr#; }'
					),
					'privacy-form-save-button-color' => array(
						'title' => esc_html__('Privacy Form Save Button Color', 'goodlayers-core'),
						'type' => 'colorpicker',
						'default' => '#1a49a8',
						'selector' => '.gdlr-core-body .gdlr-core-gdpr-form-submit input[type="submit"]{ background: #gdlr#; }'
					)
				);
			}
			
		}
	}

	add_action('init', 'gdlr_core_set_privacy_settings');
	if( !function_exists('gdlr_core_set_privacy_settings') ){
		function gdlr_core_set_privacy_settings(){
			global $gdlr_core_privacy_settings; // settings from theme option
			$gdlr_core_privacy_settings = apply_filters('gdlr_core_privacy_settings', array());

			if( !empty($gdlr_core_privacy_settings['privacy-notification-position']) && $gdlr_core_privacy_settings['privacy-notification-position'] == 'top-bar' ){
				if( !empty($gdlr_core_privacy_settings['consent-item']) ){
					add_action('gdlr_core_top_privacy_box', 'gdlr_core_privacy_box');
				}
			}

			global $gdlr_core_privacy_cookie;

			if( !empty($_POST['gdlr-core-privacy-settings']) ){
				$data = array();
				$count = 0;
				foreach( $gdlr_core_privacy_settings['consent-item'] as $consent ){ $count++;
					$data['gdlr-core-privacy-' . $count] = empty($_POST['gdlr-core-privacy-' . $count])? 0: 1;
				}
				$cookie_time = empty($gdlr_core_privacy_settings['privacy-cookie-time'])? 30: intval($gdlr_core_privacy_settings['privacy-cookie-time']);
				$cookie_time = $cookie_time * 86400;
				setcookie('gdlr-core-privacy-settings', json_encode($data), time() + $cookie_time, '/');
				$gdlr_core_privacy_cookie = $data;
			}else if( !empty($_COOKIE['gdlr-core-privacy-settings']) ){
				$gdlr_core_privacy_cookie = json_decode(wp_unslash($_COOKIE['gdlr-core-privacy-settings']), true);
			}

			// remove cookie
			// setcookie('gdlr-core-privacy-settings', '', 1, '/');
			// setcookie('gdlr-core-privacy-box', '', 1, '/');

		} // gdlr_core_set_privacy_settings
	}

	// add script to header/footer
	add_action('wp_head', 'gdlr_core_head_privacy_script');
	if( !function_exists('gdlr_core_head_privacy_script') ){
		function gdlr_core_head_privacy_script(){
			global $gdlr_core_privacy_settings, $gdlr_core_privacy_cookie;

			// show only header script
			if( !empty($gdlr_core_privacy_settings['consent-item']) ){
				if( !empty($_COOKIE['gdlr-core-privacy-box']) || !empty($gdlr_core_privacy_cookie) ){
					$count = 0;

					foreach( $gdlr_core_privacy_settings['consent-item'] as $consent ){ $count++;
						if( $consent['sub-title'] == 'cookie-consent' && $consent['script-type'] == 'snippet' && 
							$consent['placement'] == 'head' && !empty($consent['snippet']) ){
							
							if( isset($gdlr_core_privacy_cookie['gdlr-core-privacy-' . $count]) ){
								if( !empty($gdlr_core_privacy_cookie['gdlr-core-privacy-' . $count]) ){
									echo gdlr_core_text_filter($consent['snippet']);
								}
							}else{
								if( !empty($consent) && $consent['default'] == 'enable' ){
									echo gdlr_core_text_filter($consent['snippet']);
								} 
							}
						}
					}
				}
			}


		}
	} 
	add_action('wp_footer', 'gdlr_core_footer_privacy_script');
	if( !function_exists('gdlr_core_footer_privacy_script') ){
		function gdlr_core_footer_privacy_script(){
			global $gdlr_core_privacy_settings, $gdlr_core_privacy_cookie;

			
			if( !empty($gdlr_core_privacy_settings['consent-item']) ){
			
				// show only footer script
				if( !empty($_COOKIE['gdlr-core-privacy-box']) || !empty($gdlr_core_privacy_cookie) ){
					$count = 0;

					foreach( $gdlr_core_privacy_settings['consent-item'] as $consent ){ $count++;

						if( $consent['sub-title'] == 'cookie-consent' && $consent['script-type'] == 'snippet' && 
							$consent['placement'] == 'footer' && !empty($consent['snippet']) ){

							if( isset($gdlr_core_privacy_cookie['gdlr-core-privacy-' . $count]) ){
								if( !empty($gdlr_core_privacy_cookie['gdlr-core-privacy-' . $count]) ){
									echo gdlr_core_text_filter($consent['snippet']);
								}
							}else{
								if( !empty($consent) && $consent['default'] == 'enable' ){
									echo gdlr_core_text_filter($consent['snippet']);
								} 
							}
						}
					}
				}

				$privacy_box_pos = empty($gdlr_core_privacy_settings['privacy-notification-position'])? 'bottom-bar': $gdlr_core_privacy_settings['privacy-notification-position'];
				if( $privacy_box_pos != 'top-bar' && $privacy_box_pos != 'none' ){
					gdlr_core_privacy_box($privacy_box_pos);
				}

?>
<div class="gdlr-core-lightbox-wrapper" id="gdlr-core-gdpr-lightbox" >
	<div class="gdlr-core-lightbox-row">
		<div class="gdlr-core-lightbox-cell">
			<div class="gdlr-core-lightbox-content">
				<form id="gdlr-core-gdpr-form" class="gdlr-core-js clearfix" method="POST" >
					<div class="gdlr-core-gdpr-form-left" >
						<h3 class="gdlr-core-gdlr-form-left-title" ><?php 
							if( empty($gdlr_core_privacy_settings['privacy-form-logo']) ){
								esc_html_e('GDPR', 'goodlayers-core'); 
							}else{
								echo gdlr_core_get_image($gdlr_core_privacy_settings['privacy-form-logo']);
							}
						?></h3> 
						<ul class="gdlr-core-gdlr-form-nav">
						<?php
							$count = 0;
							foreach( $gdlr_core_privacy_settings['consent-item'] as $consent ){ $count++;
								if( !empty($consent['title']) ){
									echo '<li class="' . ($count == 1? 'gdlr-core-active': '') . '" data-gdlr-nav="' . esc_attr($count) . '" >' . gdlr_core_text_filter($consent['title']) . '</li>';
								}
							}
						?>
						</ul>
					</div>
					<div class="gdlr-core-gdpr-form-right" >
						<?php 
							$count = 0;
							foreach( $gdlr_core_privacy_settings['consent-item'] as $consent ){ $count++;
								echo '<div class="gdlr-core-gdpr-form-content-wrap ' . ($count == 1? 'gdlr-core-active': '') . '" data-gdlr-nav="' . esc_attr($count) . '" >';
								if( !empty($consent['title']) ){
									echo '<h3 class="gdlr-core-gdpr-form-title">' . gdlr_core_text_filter($consent['title']) . '</h3>';
								}
								if( !empty($consent['content']) ){
									echo '<div class="gdlr-core-gdpr-form-content" >' . gdlr_core_content_filter($consent['content']) . '</div>';
								}

								if( !empty($consent['sub-title']) && $consent['sub-title'] == 'cookie-consent' ){
									echo '<div class="gdlr-core-gdpr-form-checkbox-item" >';
									echo '<label>';
									if( !empty($consent['default']) && $consent['default'] == 'enable' &&
										!empty($consent['required']) && $consent['required'] == 'enable' ){

										echo '<input type="hidden" class="gdlr-core-gdpr-form-checkbox-required" name="gdlr-core-privacy-' . $count . '" value="1" />';
									}else{
										echo '<input type="checkbox" class="gdlr-core-gdpr-form-checkbox" name="gdlr-core-privacy-' . $count . '" ';
										if( isset($gdlr_core_privacy_cookie['gdlr-core-privacy-' . $count]) ){
											if( !empty($gdlr_core_privacy_cookie['gdlr-core-privacy-' . $count]) ){
												echo 'checked';
											}
										}else{
											if( !empty($consent['default']) && $consent['default'] == 'enable' ){
												echo 'checked';
											}
										}
										echo ' />';
									}

									echo '<div class="gdlr-core-gdpr-form-checkbox-appearance" >';
									echo '<div class="gdlr-core-gdpr-form-checkbox-button" ></div>';
									echo '</div>'; // gdlr-core-gdpr-form-checkbox-appearance

									echo '<div class="gdlr-core-gdpr-form-checkbox-text">';
									echo '<span class="gdlr-core-enable" >' . esc_html__('Enable', 'goodlayers-core') . '</span>';
									echo '<span class="gdlr-core-enable-required" >' . esc_html__('Enable (Required)', 'goodlayers-core') . '</span>';
									echo '<span class="gdlr-core-disable" >' . esc_html__('Disable', 'goodlayers-core') . '</span>';
									echo '</div>';
									echo '</label>';
									echo '</div>'; // gdlr-core-gdpr-form-checkbox-item
								}
								echo '</div>'; // gdlr-core-gdpr-form-content-wrap
							}
						?>
						<div class="gdlr-core-gdpr-form-submit" >
							<input type="hidden" name="gdlr-core-privacy-settings" value="1" /> 
							<input type="submit" value="<?php esc_html_e('Save Settings', 'goodlayers-core'); ?>" />
						</div>
					</div><!-- gdpr-form-right -->
				</form>
				<div class="gdlr-core-lightbox-form-close"></div>
			</div>
		</div>
	</div>
</div>
<?php
			}

		} // gdlr_core_footer_privacy_script
	} 

	if( !function_exists('gdlr_core_youtube_cookies') ){
		function gdlr_core_youtube_cookies(){
			global $gdlr_core_privacy_settings;
			if( !empty($gdlr_core_privacy_settings['disable-cookie-youtube']) && $gdlr_core_privacy_settings['disable-cookie-youtube'] == 'enable' ){
				return false;
			}

			return true;
		} // gdlr_core_youtube_cookies
	}

	if( !function_exists('gdlr_core_privacy_box') ){
		function gdlr_core_privacy_box( $position = '' ){
			global $gdlr_core_privacy_cookie, $gdlr_core_privacy_settings;

			if( !empty($_COOKIE['gdlr-core-privacy-box']) || !empty($gdlr_core_privacy_cookie) ){
				return '';
			}

			$position = empty($position)? 'top-bar': $position;

			echo '<div class="gdlr-core-privacy-box-wrap gdlr-core-pos-' . esc_attr($position) . ' clearfix" >';
			echo '<div class="gdlr-core-privacy-box-text" >';
			echo sprintf(
					esc_html__('We are using cookies to give you the best experience. You can find out more about which cookies we are using or switch them off in %s.', 'goodlayers-core'), 
					'<a href="#" class="gdlr-core-privacy-box-lb gdlr-core-js" data-gdlr-lb="gdlr-core-gdpr-lightbox" >' . esc_html__('privacy settings', 'goodlayers-core') . '</a>'
				);
			echo '</div>';

			echo '<div class="gdlr-core-privacy-box-action" >';
			// echo '<input type="hidden" name="gdlr-core-privacy-accept-all" value="1" />';
			// echo '<input type="submit" class="gdlr-core-privacy-box-button" value="' . esc_attr__('Accept', 'goodlayers-core') . '" >';
			$cookie_time = empty($gdlr_core_privacy_settings['privacy-cookie-time'])? 30: intval($gdlr_core_privacy_settings['privacy-cookie-time']);
			$cookie_time = $cookie_time * 86400;
			echo '<a href="#" class="gdlr-core-privacy-box-button gdlr-core-js" ';
			echo 'data-cookie-time="' . esc_attr($cookie_time) . '" ';
			echo ' >' . esc_html__('Accept', 'goodlayers-core') . '</a>';
			echo '<a href="#" class="gdlr-core-privacy-box-lb gdlr-core-js" data-gdlr-lb="gdlr-core-gdpr-lightbox" >' . esc_html__('Privacy Settings', 'goodlayers-core') . '</a>';
			echo '</div>'; // gdlr-core-privacy-box-action
			echo '</div>'; // gdlr-core-privacy-box-wrap
		}
	}

	if( !function_exists('gdlr_core_get_pb_privacy_options') ){
		function gdlr_core_get_pb_privacy_options(){
			global $gdlr_core_privacy_settings;

			$ret = array(
				'' => esc_html__('None', 'goodlayers-core')
			);

			if( !empty($gdlr_core_privacy_settings['consent-item']) ){
				foreach( $gdlr_core_privacy_settings['consent-item'] as $consent ){
					if( $consent['sub-title'] == 'cookie-consent' && $consent['script-type'] == 'page-item-consent' &&
						!empty($consent['title']) ){

						$ret[$consent['title']] = $consent['title'];
					}
				}
			}
			return $ret;
		}
	}

	if( !function_exists('gdlr_core_get_pb_privacy_box') ){
		function gdlr_core_get_pb_privacy_box( $consent_title, $style = 'pb-item', $settings = array() ){
			global $gdlr_core_privacy_settings, $gdlr_core_privacy_cookie;

			// find the consent id by title
			$count = 0;
			$consent_id = 0;
			foreach( $gdlr_core_privacy_settings['consent-item'] as $consent ){ $count++;
				if( $consent['title'] == $consent_title ){
					$consent_id = $count;
				}
			}

			// check and return consent
			$show_consent_box = true;
			if( !empty($_COOKIE['gdlr-core-privacy-box']) || !empty($gdlr_core_privacy_cookie) ){
				if( empty($consent_id) ){
					$show_consent_box = false;
				}else if( isset($gdlr_core_privacy_cookie['gdlr-core-privacy-' . $consent_id]) ){
					if( !empty($gdlr_core_privacy_cookie['gdlr-core-privacy-' . $consent_id]) ){
						$show_consent_box = false;
					}
				}else{
					if( $gdlr_core_privacy_settings['consent-item'][$consent_id - 1]['default'] == 'enable' ){
						$show_consent_box = false;
					} 
				}
			}

			if( $show_consent_box ){
				$ret  = '<div class="gdlr-core-pb-privacy-box-wrap gdlr-core-type-' . esc_attr($style) . (empty($settings['wrapper-class'])? '': ' ' . esc_attr($settings['wrapper-class'])) . '" ';
				$ret .= empty($settings['wrapper-css'])? '': gdlr_core_esc_style($settings['wrapper-css']);
				$ret .= empty($settings['sync-height'])? '': ' data-sync-height="' . esc_attr($settings['sync-height']) . '" ';
				$ret .= '>';
				if( $style == 'pb-item' ){
					$ret .= '<img src="' . GDLR_CORE_URL . '/include/images/content-blocked.jpg" alt="lock" />';
				}
				$ret .= '<div class="gdlr-core-pb-privacy-box-overlay" ></div>';
				$ret .= '<div class="gdlr-core-pb-privacy-box-content-wrap" >';
				$ret .= '<div class="gdlr-core-pb-privacy-box-content-table" >';
				$ret .= '<div class="gdlr-core-pb-privacy-box-content-cell" >';
				$ret .= '<h3 class="gdlr-core-pb-privacy-box-title" >' . esc_html__('This content is blocked', 'goodlayers-core') . '</h3>';
				$ret .= '<div class="gdlr-core-pb-privacy-box-content" >';
				$ret .= sprintf(
					esc_html__('If you want to access this content, please review your %s', 'goodlayers-core'),
					'<a href="#" class="gdlr-core-js" data-gdlr-lb="gdlr-core-gdpr-lightbox" >' . esc_html__('Cookie Consent Settings', 'goodlayers-core') . '</a>'
				);
				$ret .= '</div>'; // gdlr-core-pb-privacy-box-content
				$ret .= '</div>'; // gdlr-core-pb-privacy-box-content-cell
				$ret .= '</div>'; // gdlr-core-pb-privacy-box-content-table
				$ret .= '</div>'; // gdlr-core-pb-privacy-box-content-wrap
				$ret .= '</div>'; // gdlr-core-pb-privacy-box-wrap
			}else{
				$ret  = false;
			}

			return $ret;
		}
	}