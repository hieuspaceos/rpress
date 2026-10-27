<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('audio', 'gdlr_core_pb_element_audio'); 
	
	if( !class_exists('gdlr_core_pb_element_audio') ){
		class gdlr_core_pb_element_audio{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-music',
					'title' => esc_html__('Audio', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(	
							'audio-url' => array(
								'title' => esc_html__('Audio URL', 'goodlayers-core'),
								'type' => 'text',
								'default' => ''
							),			
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(	
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),				
						)
					),
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-audio-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-audio-<?php echo esc_attr($id); ?>').parent().gdlr_core_mejs_ajax();
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
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				// start printing item
				$extra_class  = empty($settings['class'])? '': $settings['class'];
				$ret  = '<div class="gdlr-core-audio-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( !empty($settings['audio-url']) ){
					$ret .= gdlr_core_get_audio($settings['audio-url']);
				}else{
					$ret .= '<div class="gdlr-core-external-plugin-message">' . esc_html__('Please upload and fill the audio url to display the audio player.', 'goodlayers-core') . '</div>';
				}

				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_audio
	} // class_exists	