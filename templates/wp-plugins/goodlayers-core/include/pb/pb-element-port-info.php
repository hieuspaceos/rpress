<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('port-info', 'gdlr_core_pb_element_port_info'); 
	
	if( !class_exists('gdlr_core_pb_element_port_info') ){
		class gdlr_core_pb_element_port_info{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'icon_briefcase',
					'title' => esc_html__('Portfolio Info', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(

							'port-info' => array(
								'title' => esc_html__('Portfolio Info', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'key-value',
								'default' => array(
									array(
										'key'=>esc_html__('Client', 'goodlayers-core'),
										'value'=>esc_html__('Sample Client', 'goodlayers-core'),
									),
									array(
										'key'=>esc_html__('Skills', 'goodlayers-core'),
										'value'=>esc_html__('Sample Skill, Sample Skill 2', 'goodlayers-core'),
									),
									array(
										'key'=>esc_html__('Website', 'goodlayers-core'),
										'value'=>esc_html__('Goodlayers.com', 'goodlayers-core'),
									),
								),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'enable-post-type-tax' => array(
								'title' => esc_html__('Enable Post Type Taxonomy', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
							),
							'enable-bottom-border' => array(
								'title' => esc_html__('With Bottom Border', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
							),
							'taxonomy-head-text' => array(
								'title' => esc_html__('Taxonomy Head Text', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Tags', 'goodlayers-core'),
								'condition' => array('enable-post-type-tax' => 'enable')
							),
							'taxonomy' => array(
								'title' => esc_html__('Select Taxonomy', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => gdlr_core_get_taxonomies(),
								'condition' => array('enable-post-type-tax' => 'enable'),
								'default' => 'portfolio_tag'
							)	

						)
					), // general
					'social-share' => array(
						'title' => esc_html__('Social Share', 'goodlayers-core'),
						'options' => array(

							'enable-social-share' => array(
								'title' => esc_html__('Enable Social Share', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'default' => esc_html__('Default', 'goodlayers-core'),
									'enable' => esc_html__('Enable', 'goodlayers-core'),
									'disable' => esc_html__('Disable', 'goodlayers-core'),
								),
								'default' => 'default',
								'description' => esc_html__('Set this option to default to display the social share based on the option set at the theme option area.', 'goodlayers-core')
							),
							'social-facebook' => array(
								'title' => esc_html__('Facebook', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array('enable-social-share' => 'enable')
							),			
							'social-linkedin' => array(
								'title' => esc_html__('Linkedin', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array('enable-social-share' => 'enable')
							),				
							'social-pinterest' => array(
								'title' => esc_html__('Pinterest', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array('enable-social-share' => 'enable')
							),			
							'social-stumbleupon' => array(
								'title' => esc_html__('Stumbleupon', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array('enable-social-share' => 'enable')
							),			
							'social-twitter' => array(
								'title' => esc_html__('Twitter', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array('enable-social-share' => 'enable')
							),			
							'social-email' => array(
								'title' => esc_html__('Email', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array('enable-social-share' => 'enable')
							),

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
?><script id="gdlr-core-preview-text-box-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-text-box-<?php echo esc_attr($id); ?>').parent().gdlr_core_content_script();
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
						'port-info' => array(
							array(
								'key'=>esc_html__('Client', 'goodlayers-core'),
								'value'=>esc_html__('Sample Client', 'goodlayers-core'),
							),
							array(
								'key'=>esc_html__('Skills', 'goodlayers-core'),
								'value'=>esc_html__('Sample Skill, Sample Skill 2', 'goodlayers-core'),
							),
							array(
								'key'=>esc_html__('Website', 'goodlayers-core'),
								'value'=>esc_html__('Goodlayers.com', 'goodlayers-core'),
							),
						),
						'text-align' => 'left',
						'enable-post-type-tax' => 'disable',
						'enable-social-share' => 'default',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$extra_class  = empty($settings['no-pdlr'])? ' gdlr-core-item-pdlr': '';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$ret  = '<div class="gdlr-core-port-info-item gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				// tab list
				$extra_class = (empty($settings['enable-bottom-border']) || $settings['enable-bottom-border'] == 'enable')? ' gdlr-core-with-border': '';
				$ret .= '<div class="gdlr-core-port-info-wrap gdlr-core-skin-divider ' . esc_attr($extra_class) . '" >';
				if( !empty($settings['port-info']) ){
					foreach( $settings['port-info'] as $tab ){
						$ret .= '<div class="gdlr-core-port-info" >';
						if( !empty($tab['key']) ){
							$ret .= '<span class="gdlr-core-port-info-key gdlr-core-skin-title" >' . gdlr_core_text_filter($tab['key']) . '</span>';
						}
						if( !empty($tab['value']) ){
							$ret .= '<span class="gdlr-core-port-info-value" >' . gdlr_core_text_filter($tab['value']) . '</span>';
						}
						$ret .= '</div>';
					}
				}

				// taxonomy
				if( !empty($settings['enable-post-type-tax']) && $settings['enable-post-type-tax'] == 'enable' ){

					if( !empty($settings['taxonomy']) ){
						$post_id = (is_admin() && !empty($_GET['post']))? $_GET['post']: get_the_ID();
						$tag = get_the_term_list($post_id, $settings['taxonomy'], '', '<span class="gdlr-core-sep">,</span> ' , '');

						if( !empty($tag) ){
							$ret .= '<div class="gdlr-core-port-info gdlr-core-port-info-post-type-tax" >';
							if( !empty($settings['taxonomy-head-text']) ){
								$ret .= '<span class="gdlr-core-port-info-key gdlr-core-skin-title" >' . gdlr_core_text_filter($settings['taxonomy-head-text']) . '</span>';
							}
							$ret .= '<span class="gdlr-core-port-info-value" >' . $tag . '</span>';
							$ret .= '</div>';
						}
					}
					
				}

				// social section
				if( !empty($settings['enable-social-share']) && $settings['enable-social-share'] != 'disable' ){
					$social_list = array(
						'layout'=>'left-text', 
						'text-align'=>'center',
						'facebook'=>empty($settings['social-facebook'])? 'enable': $settings['social-facebook'],
						'linkedin'=>empty($settings['social-linkedin'])? 'disable': $settings['social-linkedin'],
						'pinterest'=>empty($settings['social-pinterest'])? 'enable': $settings['social-pinterest'],
						'stumbleupon'=>empty($settings['social-stumbleupon'])? 'disable': $settings['social-stumbleupon'],
						'twitter'=>empty($settings['social-twitter'])? 'enable': $settings['social-twitter'],
						'email'=>empty($settings['social-email'])? 'disable': $settings['social-email'],
					);

					if( $settings['enable-social-share'] == 'default' ){
						$social_list = apply_filters('gdlr_core_social_share_list', $social_list);
					}

					$social_list['layout'] = 'left-text';
					$social_list['text-align'] = 'left';
					$social_list['social-head'] = 'none';
					$social_list['padding-bottom'] = '0px';
					$ret .= '<div class="gdlr-core-port-info gdlr-core-port-info-social-share gdlr-core-skin-divider" >';
					$ret .= '<span class="gdlr-core-port-info-key gdlr-core-skin-title" >' . esc_html__('Share', 'goodlayers-core') . '</span>';
					$ret .= '<div class="gdlr-core-port-info-value" >' . gdlr_core_pb_element_social_share::get_content($social_list) . '</div>';
					$ret .= '</div>';
				}

				$ret .= '</div>'; // gdlr-core-port-info-wrap


				$ret .= '</div>';

				return $ret;
			}
			
		} // gdlr_core_pb_element_port_info
	} // class_exists	

	// [gdlr_core_port_info]
	// [gdlr_core_tab title="key" ]value[/gdlr_core_tab]
	// [gdlr_core_tab title="key" ]value[/gdlr_core_tab]
	// [gdlr_core_tab title="key" ]value[/gdlr_core_tab]
	// [/gdlr_core_port_info]
	add_shortcode('gdlr_core_port_info', 'gdlr_core_port_info_shortcode');
	if( !function_exists('gdlr_core_port_info_shortcode') ){
		function gdlr_core_port_info_shortcode($atts, $content = ''){
			$atts = wp_parse_args($atts, array(
				'no-pdlr' => true,
				'enable-bottom-border' => 'disable'
			));

			global $gdlr_core_tabs;
			$gdlr_core_tabs = array();

			do_shortcode($content);

			$port_info = array();
			foreach( $gdlr_core_tabs as $tab ){
				$port_info[] = array(
					'key' => $tab['title'],
					'value' => $tab['content'],
				);
			}
			$atts['port-info'] = $port_info;


			return gdlr_core_pb_element_port_info::get_content($atts);
		}
	}	

	add_shortcode('gdlr_core_port_info2', 'gdlr_core_port_info2_shortcode');
	if( !function_exists('gdlr_core_port_info2_shortcode') ){
		function gdlr_core_port_info2_shortcode($atts, $content = ''){
			$atts = wp_parse_args($atts, array());

			global $gdlr_core_tabs;
			$gdlr_core_tabs = array();

			do_shortcode($content);

			$ret  = '<div class="gdlr-core-port-info2-item" >';
			foreach( $gdlr_core_tabs as $tab ){
				$ret .= '<div class="gdlr-core-port-info2" >';
				$ret .= '<div class="gdlr-core-port-info2-content clearfix" ' . gdlr_core_esc_style(array(
					'max-width' => empty($atts['max-width'])? '': $atts['max-width']
				)) . ' >';
				if( !empty($tab['title']) ){
					$ret .= '<span class="gdlr-core-port-info2-key gdlr-core-skin-title" >' . gdlr_core_text_filter($tab['title']) . '</span>';
				}
				if( !empty($tab['content']) ){
					$ret .= '<span class="gdlr-core-port-info2-value" >' . gdlr_core_text_filter($tab['content']) . '</span>';
				}
				$ret .= '</div>'; // gdlr-core-port-info2-content
				$ret .= '</div>'; // gdlr-core-port-info2
			}
			$ret .= '</div>';


			return $ret;
		}
	}