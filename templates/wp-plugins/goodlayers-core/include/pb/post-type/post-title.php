<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('post-title', 'gdlr_core_pb_post_title'); 
	
	if( !class_exists('gdlr_core_pb_post_title') ){
		class gdlr_core_pb_post_title{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-text-width',
					'title' => esc_html__('Post Title', 'goodlayers-core'),
					'type' => array('post_type'),
					'nowrap' => true
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'heading-tag' => array(
								'title' => esc_html__('Heading Tag', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'h1' => 'h1',
									'h2' => 'h2',
									'h3' => 'h3',
									'h4' => 'h4',
									'h5' => 'h5',
									'h6' => 'h6'
								),
								'default' => 'h3'
							),
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
				
				$settings = empty($settings)? array(): $settings;
				$settings['heading-tag'] = empty($settings['heading-tag'])? 'h3': $settings['heading-tag'];
				
				$ret  = '<' . $settings['heading-tag'] . '>';
				if( $preview ){
					$ret .= esc_html__('Sample Post Title', 'goodlayers-core'); 
				}else{	
					$ret .= get_the_title();
				}
				$ret .= '</' . $settings['heading-tag'] . '>';

				return $ret;
			}
			
		} // gdlr_core_pb_element_title
	} // class_exists	

	// [gdlr_core_title title="" caption="" ]
	add_shortcode('gdlr_core_title', 'gdlr_core_title_shortcode');
	if( !function_exists('gdlr_core_title_shortcode') ){
		function gdlr_core_title_shortcode($atts, $content = ''){
			$atts = wp_parse_args($atts, array(
				'no-pdlr' => true
			));

			return gdlr_core_pb_element_title::get_content($atts);
		}
	}