<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	if( !class_exists('gdlr_core_pb_wrapper_flexbox') ){
		class gdlr_core_pb_wrapper_flexbox{
			
			// get the element settings
			static function get_settings(){
				return array(	
					'type' => 'flexbox',
					'title' => esc_html__('Flexbox', 'goodlayers-core'),
					'thumbnail' => GDLR_CORE_URL . '/framework/images/page-builder/wrapper.png',
				);
			}
			
			// return the element options
			static function get_options(){
				$options = array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'gap' => array(
								'title' => esc_html__('Gap', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'justify-content' => array(
								'title' => esc_html__('Justify Content', 'goodlayers-core'),
								'type' => 'text',
							),
							'align-items' => array(
								'title' => esc_html__('Align Items', 'goodlayers-core'),
								'type' => 'text',
							),
						),
					)

				);

				return $options;
			}			
			
			// get element template for page builder
			static function get_template( $options = array(), $callback = '' ){

				$flex_atts = array(
					'gap' => empty($options['value']['gap'])? '': $options['value']['gap'],
					'justify-content' => empty($options['value']['justify-content'])? '': $options['value']['justify-content'],
					'align-items' => empty($options['value']['align-items'])? '': $options['value']['align-items'],
				);
	
				$wrapper  = '<div class="gdlr-core-page-builder-flexbox" data-template="wrapper" data-type="flexbox" ';
				$wrapper .= (empty($options['value'])? '': 'data-value="' . esc_attr(json_encode($options['value'])) . '" ');
				$wrapper .= '>';
				
				$wrapper .= '<div class="gdlr-core-page-builder-flexbox-content" >';
				$wrapper .= '<div class="gdlr-core-page-builder-flexbox-head">';
				$wrapper .= '<span class="gdlr-core-page-builder-flexbox-head-title" >' . esc_html__('Flexbox', 'goodlayers-core') . '</span>';
				$wrapper .= '<div class="gdlr-core-page-builder-flexbox-settings">';
				$wrapper .= '<i class="fa fa-edit gdlr-core-page-builder-flexbox-edit" ></i>';
				$wrapper .= '<i class="fa fa-copy gdlr-core-page-builder-flexbox-copy" ></i>';
				$wrapper .= '<i class="fa fa-remove gdlr-core-page-builder-flexbox-remove" ></i>';
				$wrapper .= '</div>'; // gdlr-core-page-builder-flexbox-settings
				$wrapper .= '</div>'; // gdlr-core-page-builder-flexbox-head
				
				$wrapper_display_class  = empty($options['value']['class'])? '': $options['value']['class'];

				$wrapper .= '<div class="gdlr-core-page-builder-flexbox-content-margin" >';
				$wrapper .= '<div class="gdlr-core-page-builder-flexbox-content-wrap gdlr-core-pbf-flexbox-content gdlr-core-js" >';
				$wrapper .= '<div class="gdlr-core-page-builder-flexbox-container gdlr-core-flexbox-sortable" ' . gdlr_core_esc_style($flex_atts) . ' >';
				if( !empty($options['items']) ){
					$wrapper .= gdlr_core_escape_content(call_user_func($callback, $options['items']));
				}
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-container
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-content-wrap
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-content-margin
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-content

				// script for background wrapper
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-background-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	var background_wrap = jQuery('#gdlr-core-preview-background-<?php echo esc_attr($id); ?>').parent();
	background_wrap.gdlr_core_parallax_background().gdlr_core_particle_background().gdlr_core_marquee().gdlr_core_fluid_video().gdlr_core_ux();
	jQuery(window).trigger('scroll');
});
</script><?php	
				$wrapper .= ob_get_contents();
				ob_end_clean();				
				
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper
				
				return $wrapper;
				
			}
			
			// get element content for front end page builder
			static function get_content( $options = array(), $callback = '' ){

				$flex_atts = array(
					'gap' => empty($options['value']['gap'])? '': $options['value']['gap'],
					'justify-content' => empty($options['value']['justify-content'])? '': $options['value']['justify-content'],
					'align-items' => empty($options['value']['align-items'])? '': $options['value']['align-items'],
				);

				$wrapper  = '<div class="gdlr-core-pbf-flexbox" ' . gdlr_core_esc_style($flex_atts) . ' >';
				if( !empty($options['items']) ){
					$wrapper .= gdlr_core_escape_content(call_user_func($callback, $options['items']));
				}
				$wrapper .= '</div>'; // gdlr-core-pbf-wrapper

				return $wrapper;
				
			}
			
		} // gdlr_core_pb_wrapper_background
	} // class_exists	